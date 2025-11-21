# Fonctionnalités manquantes dans generator

Comparaison entre `codegen` (v1) et `generator` pour identifier ce qui reste à implémenter.

## ✅ Implémenté dans v2

- ✅ **String** - Ressources string basiques
- ✅ **Number** - Nombres (i64/f64 basiques)
- ✅ **Bool** - Booléens
- ✅ **Namespaces** - Support des namespaces (`<ns name="...">`)
- ✅ **Profiles** - Filtrage par profile (debug/release)
- ✅ **Tests resources** - Support de `res/tests/`
- ✅ **Validation doublons** - Détection des clés dupliquées
- ✅ **Génération de code** - Module `r::` avec structure de namespace

## ❌ Manquant dans v2

### Types de ressources

1. **Color** (`<color>`) - Valeurs hexadécimales (#FF5722, #AAFF5722)
2. **Url** (`<url>`) - URLs
3. **Dimension** (`<dimension>`) - Valeurs avec unités (16dp, 24px, 1.5em)
4. **StringArray** (`<string-array>`) - Tableaux de strings
5. **IntArray** (`<int-array>`) - Tableaux d'entiers
6. **FloatArray** (`<float-array>`) - Tableaux de flottants

### Fonctionnalités avancées

7. **References** (`@string/app_name`) - Références vers d'autres ressources
8. **InterpolatedString** (`"Welcome to @string/app_name!"`) - Strings avec références interpolées
9. **Templates** (`<string template="Hello {name}">`) - Strings avec paramètres
10. **Validation des références** - Vérifier que toutes les références existent
11. **NumberValue avancé** :
    - `BigDecimal` - Nombres avec précision arbitraire
    - Types explicites (`type="i32"`, `type="u32"`, etc.) - Forcer un type Rust spécifique
12. **Module r_tests** - Génération du module `r_tests::` pour les ressources de test
13. **Résolution de références** - Génération de `pub use` pour les références

### Autres

14. **Génération de code vide** - Quand `res/` n'existe pas
15. **Support des attributs `profile`** - Filtrage au niveau parsing (déjà fait dans preprocessing)

## Priorités suggérées

### Haute priorité
- Arrays (string/int/float) - Très utilisés
- References - Fonctionnalité importante pour la réutilisation
- Validation des références - Essentiel pour la sécurité

### Moyenne priorité
- Color, Url, Dimension - Types spécialisés utiles
- InterpolatedString - Améliore l'expressivité
- Module r_tests - Pour les tests

### Basse priorité
- Templates - Fonctionnalité avancée
- BigDecimal - Cas d'usage spécifiques
- Types explicites pour numbers - Nice to have

## Notes d'implémentation

Avec le nouveau système modulaire (`ir/types/`), l'ajout de nouveaux types est simplifié :
- Créer `ir/types/color.rs`, `ir/types/url.rs`, etc.
- Implémenter le trait `ResourceType`
- Enregistrer dans `TypeRegistry::default()`

Pour les références et interpolations, il faudra :
- Étendre `ResourceValue` dans `ir/model.rs`
- Ajouter la logique de parsing dans `parsing/reader/handlers.rs`
- Implémenter la résolution dans `analysis/`
- Générer le code approprié dans les types concernés

