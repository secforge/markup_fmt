# `vue.customBlock`

Control how Vue custom blocks (like `<i18n>`, `<docs>`, etc.) are formatted.

Vue Single File Components (SFC) can contain custom blocks in addition to `<template>`, `<script>`, and `<style>`. These custom blocks are top-level elements used for various purposes like internationalization or documentation.

Available option values:

- `"lang-attribute"`: Use the `lang` attribute to determine formatting. Blocks with a `lang` attribute (e.g., `<i18n lang="json">`) will be formatted according to that language. Blocks without a `lang` attribute will preserve their raw content.
- `"squash"`: Format content as HTML, collapsing whitespace and inserting line breaks to fit the print width. The `lang` attribute is ignored.
- `"none"`: Never format custom block content. All content is preserved exactly as written, regardless of the `lang` attribute.

Default value is `"lang-attribute"`.

## Example for `"lang-attribute"`

Input:

```html
<template>
  <div>{{ $t('hello') }}</div>
</template>

<i18n lang="json">
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>

<i18n>
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>
```

Output:

```html
<template>
  <div>{{ $t("hello") }}</div>
</template>

<i18n lang="json">
{
  "en": { "hello": "Hello" },
  "de": { "hello": "Hallo" }
}
</i18n>

<i18n>
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>
```

Note: The first `<i18n>` block with `lang="json"` is formatted as JSON, while the second block without a `lang` attribute preserves its raw content.

## Example for `"squash"`

Input:

```html
<template>
  <div>{{ $t('hello') }}</div>
</template>

<i18n lang="json">
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>
```

Output:

```html
<template>
  <div>{{ $t("hello") }}</div>
</template>

<i18n lang="json">{"en":{"hello":"Hello" },"de": {"hello":"Hallo"}}</i18n>
```

Note: Content is formatted as HTML with whitespace collapsed, regardless of the `lang` attribute.

## Example for `"none"`

Input:

```html
<template>
  <div>{{ $t('hello') }}</div>
</template>

<i18n lang="json">
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>
```

Output:

```html
<template>
  <div>{{ $t("hello") }}</div>
</template>

<i18n lang="json">
{"en":{"hello":"Hello"  },"de":   {"hello":"Hallo"}}
</i18n>
```

Note: All custom block content is preserved exactly as written, even when a `lang` attribute is present.

## Per-Block Configuration

You can configure different formatting rules for different custom block types. This allows fine-grained control over how each custom block is handled.

### Configuration Format

In your configuration file, you can specify per-block rules under `vue.custom_block`:

```json
{
  "vue_custom_block": {
    "default": "lang-attribute",  // Optional, defaults to "lang-attribute"
    "i18n": "lang-attribute",
    "docs": "none",
    "unknown-block": "squash"
  }
}
```

Or in TOML (library format):

```toml
[vue_custom_block]
default = "lang-attribute"  # Optional, defaults to "lang-attribute"
i18n = "lang-attribute"
docs = "none"
unknown-block = "squash"
```

Or in dprint format (using vue.customBlock):

```toml
"vue.customBlock" = "langAttribute"
"vue.customBlock.i18n" = "langAttribute"
"vue.customBlock.docs" = "none"
"vue.customBlock.unknownBlock" = "squash"
```

The `default` field (or the base `vue.customBlock` key in dprint) sets the default formatting mode for all custom blocks not explicitly configured. Then, each custom block type can override this default.

### Simplified Format

If you only need to set the default mode without any per-block overrides, you can use a simplified string format.

For dprint:

```json
{
  "vue.customBlock": "langAttribute"
}
```

For JSON/library contexts:

```json
{
  "vue_custom_block": "none"
}
```

This is equivalent to:

```json
{
  "vue_custom_block": {
    "default": "none"
  }
}
```

### Important: Default Field Requirement

**For dprint plugin users**: When using the dprint plugin, you can specify only override keys without the default. The default will be `"langAttribute"` if not specified:

```json
{
  "markup": {
    "vue.customBlock.i18n": "none"
  }
}
```

**For JSON/TOML configuration files** (when using markup_fmt as a library):

Use the `vue_custom_block` key (with underscore) for the struct format:

```toml
# The default field is optional - defaults to "lang-attribute" if omitted
[vue_custom_block]
i18n = "none"
docs = "squash"

# Or with explicit default
[vue_custom_block]
default = "lang-attribute"
i18n = "none"
```

**For dprint configuration**:

Use the `vue.customBlock` key (with dot and camelCase) for the flat key format:

```toml
# Sets default for all blocks
"vue.customBlock" = "langAttribute"

# Or with per-block overrides
"vue.customBlock" = "langAttribute"
"vue.customBlock.i18n" = "none"
```

This works in dprint because it uses a different configuration parser that supports dotted keys.

### Case Insensitivity

Block names are matched case-insensitively, so `<I18N>`, `<i18n>`, and `<I18n>` will all match the `i18n` configuration.

### Example

Given this configuration:

```json
{
  "vue_custom_block": {
    "default": "lang-attribute",
    "docs": "none",
    "metadata": "squash"
  }
}
```

Input:

```html
<template>
  <div>Hello</div>
</template>

<i18n lang="json">
{"key":   "value"}
</i18n>

<docs>
  <p>This is  documentation</p>
</docs>

<metadata>
  <div>Author:     Jane Doe</div>
</metadata>
```

Output:

```html
<template>
  <div>Hello</div>
</template>

<i18n lang="json">
{
  "key": "value"
}
</i18n>

<docs>
  <p>This is  documentation</p>
</docs>

<metadata><div>Author: Jane Doe</div></metadata>
```

- `<i18n>` follows the default behavior (`lang-attribute`), so it's formatted as JSON
- `<docs>` is preserved exactly as written (mode `none`)
- `<metadata>` has whitespace collapsed (mode `squash`)

## Complete dprint Configuration Example

Here's a comprehensive dprint configuration showing various markup_fmt options including Vue custom blocks:

```json
{
  "markup": {
    "printWidth": 100,
    "indentWidth": 2,
    "useTabs": false,
    "lineBreak": "lf",
    "quotes": "double",
    "formatComments": true,
    "scriptIndent": false,
    "styleIndent": false,
    "closingBracketSameLine": false,
    "closingTagLineBreakForEmpty": "fit",
    "maxAttrsPerLine": 1,
    "whitespaceSensitivity": "css",
    "scriptFormatter": "dprint",
    "vue.customBlock": "langAttribute",
    "vue.customBlock.i18n": "langAttribute",
    "vue.customBlock.docs": "none",
    "vue.customBlock.metadata": "squash"
  }
}
```

Or a minimal configuration:

```json
{
  "markup": {
    "scriptFormatter": "dprint",
    "vue.customBlock.i18n": "none",
    "vue.customBlock.docs": "none"
  }
}
```

In the minimal example, `vue.customBlock` defaults to `"langAttribute"` and only the `i18n` and `docs` blocks have overrides.
