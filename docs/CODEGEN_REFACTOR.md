# Proposition de refonte du codegen

## Contexte actuel
- Le flux `codegen::build()` enchaîne la découverte des fichiers (`multi_file`), le pré‑filtrage par profil (`environment`), le parsing XML (`parser`), la validation des références (`references`) puis la génération (`generator::flat`) sans séparation claire entre lecture, modèle intermédiaire et émission.
- Le parseur central (`parser/mod.rs`) combine état global, gestion des namespaces, tableaux, templates et typage numérique dans un seul fichier (~250 lignes) et expose directement une structure `HashMap<String, Vec<(String, ResourceValue)>>` déjà orientée génération.
- `ResourceValue` sert à la fois d’AST, de représentation validée et de format final pour l’émetteur, ce qui rend difficiles les validations ciblées (ex. profils, références cycliques, métadonnées de provenance).
- La génération plate (`generator/flat.rs`, ~500 lignes) mélange calcul de l’arborescence, résolution de dépendances (interpolations, références), formatage des littéraux et écriture de code Rust.

## Limites principales
1. **Couplage fort** : chaque étape manipule directement la `HashMap` finale, ce qui empêche d’intercaler facilement de nouvelles passes (ex. normalisation, optimisation, instrumentation).
2. **Testabilité réduite** : il est complexe de tester isolément la résolution des références, l’expansion des templates ou la détection de doublons.
3. **Extensions coûteuses** : l’ajout de nouveaux types de ressources implique de modifier plusieurs gros fichiers (parser, générateur, références).
4. **Manque de séparation I/O vs logique** : `multi_file` mélange lecture disque, filtrage par profil et agrégation.

## Objectifs de découpage
- Séparer clairement les **couches** : lecture → parsing → IR → validations → génération.
- Introduire une **IR structurée** (types dédiés) afin de capturer les métadonnées nécessaires aux passes suivantes.
- Faciliter l’ajout de nouveaux types de ressources ou de nouveaux backends (ex. génération non flat, JSON, etc.).
- Réduire la taille des fichiers critiques (<200 lignes) et clarifier les responsabilités.

## Architecture proposée

### Vue d’ensemble des couches
1. `input`: découverte des fichiers, lecture, preprocessing profil.
2. `parsing`: transformation XML → AST typé (par fichier), sans fusion.
3. `ir`: fusion multi‑fichiers + normalisation (namespaces, profils) vers une IR immutable (`ResourceGraph`).
4. `analysis`: validations (doublons, références, cycles, profils/tests).
5. `generation`: construction d’une représentation côté output puis rendu vers Rust (ou autres cibles futur).

### Nouveau découpage des modules
| Nouveau module | Responsabilité principale | Fichiers proposés |
| -------------- | ------------------------ | ----------------- |
| `codegen/input` | Recherche des fichiers, lecture, `preprocess_xml` | `input/mod.rs`, `input/scanner.rs`, `input/profile.rs` (extraction de `multi_file` + `environment`) |
| `codegen/parsing` | Parse XML → `ParsedResourceFile` (par fichier) avec événements découplés | `parsing/mod.rs`, `parsing/state.rs`, `parsing/nodes/*.rs` pour string, arrays, templates, etc. |
| `codegen/ir` | Fusion et normalisation en `ResourceGraph`, stockage des métadonnées (origine fichier, profil, namespace) | `ir/mod.rs`, `ir/resource.rs`, `ir/namespace.rs` |
| `codegen/analysis` | Validations (références, doublons, profils incompatibles) + rapports d’erreur structurés | `analysis/references.rs`, `analysis/duplicates.rs`, `analysis/report.rs` |
| `codegen/generator` | Transformations IR → `OutputModel` puis rendu (actuel `flat` devient `generator/flat/emitter.rs`, `generator/flat/tree.rs`) | `generator/mod.rs`, `generator/flat/` |
| `codegen/shared` | Types transverses (`ResourceKind`, `ScalarValue`, `ArrayValue`, utils de nommage) | `shared/types.rs`, `shared/utils.rs` |

### Détails clés
- **IR dédiée** : remplacer l’alias `HashMap<String, Vec<(String, ResourceValue)>>` par une structure type:
  ```rust
  struct ResourceGraph {
      nodes: BTreeMap<ResourceKey, ResourceNode>,
      namespaces: NamespaceTree,
  }
  struct ResourceNode {
      kind: ResourceKind,
      value: ResourcePayload,
      origin: ResourceOrigin, // fichier, ligne, profil
  }
  ```
  Cela permet de stocker provenance, statut (test/prod) et d’éviter les clones intempestifs.
- **Parsing par responsabilités** : extraire les handlers (`basic.rs`, `advanced.rs`, `arrays.rs`, `templates.rs`) en petits modules branchés via traits (ex. `impl NodeHandler for StringNode`), ce qui facilitera l’ajout de `plural`, `format`, etc.
- **Génération en deux temps** : (1) transformer `ResourceGraph` en `FlatModule` (arbre namespace + symboles résolus), (2) rendre le code (actuel `emit_*`). On pourra ensuite brancher un générateur alternatif (par exemple `generator/hierarchical.rs`).
- **Validation modulaire** : `analysis::references` ne dépendra que de l’IR et pourra être testée avec des fixtures en mémoire. La détection de cycles/interpolations pourra vivre ici.
- **Préprocessing profile/tests** : déplacer la logique `include_tests` et `PROFILE` dans `input::plan` pour exposer un `BuildPlan` clair à `build_with_options`.

## Étapes de migration suggérées
1. **Créer l’IR** : introduire `ResourceGraph` et adapter `generator::flat` pour lire cette nouvelle structure (adapter via conversion temporaire).
2. **Isoler le loader** : extraire de `multi_file` la découverte/lecture dans `input::scanner`, conserver pour l’instant l’ancienne structure de données.
3. **Refactor parser** : déplacer `ParserState` dans `parsing/state.rs`, transformer les handlers en traits/fonctions indépendantes.
4. **Brancher l’IR** : modifier la fusion multi‑fichiers pour produire des `ResourceNode` enrichis (namespace, profil, provenance).
5. **Extraire les validations** : déplacer `references::validate_references` vers `analysis::references` en travaillant sur l’IR.
6. **Nettoyer generator** : découper `flat.rs` en `tree.rs` (construction) + `emit.rs` (string builder), puis introduire un `OutputModel`.
7. **Finaliser build pipeline** : `build_with_options` orchestre `input -> parsing -> ir -> analysis -> generation` avec rapports d’erreurs uniformes.

## Questions ouvertes
- Souhaite-t-on supporter d’autres formats que XML (JSON, YAML) à moyen terme ? Si oui, l’IR devient encore plus pertinente comme pivot.
- Le support des profils/tests doit-il permettre plusieurs profils en parallèle (ex. `debug`, `release`, `bench`) ? Cela orienterait `ResourceGraph` vers des variantes par profil.
- Faut-il prévoir un backend `no_std` / `const fn` plus strict ? Ce point influencera la conception du générateur.

## Conclusion
Ce découpage garde la logique existante mais clarifie les responsabilités. Il permet d’insérer facilement de nouvelles passes (optimisations, linting), de cibler les tests unitaires et d’ouvrir la voie à d’autres sorties ou pipelines (ex. export JSON pour tooling). Les étapes ci‑dessus peuvent être menées de façon incrémentale sans casser l’API publique. 

