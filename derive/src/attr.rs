use syn::{Attribute, Meta, LitStr, Result};

/// Parsed representation of a single field's #[i18n(...)] attribute.
#[derive(Default)]
pub struct FieldI18nAttr {
    pub key: Option<String>,
    pub skip: bool,
}

/// Parsed representation of struct-level #[i18n(...)] attributes.
#[derive(Default)]
pub struct StructI18nConfig {
    pub namespace: Option<String>,
}

/// Parse struct-level #[i18n(namespace = "...")] attribute.
pub fn parse_struct_attrs(attrs: &[Attribute]) -> Result<StructI18nConfig> {
    let mut config = StructI18nConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("i18n") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("namespace") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                config.namespace = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error("unsupported struct-level i18n attribute"))
            }
        })?;
    }
    Ok(config)
}

/// Parse a field's #[i18n], #[i18n(key = "...")], or #[i18n(skip)].
/// Returns Some(attr) if the field has an i18n attribute, None otherwise.
pub fn parse_field_attrs(attrs: &[Attribute]) -> Result<Option<FieldI18nAttr>> {
    let mut result = FieldI18nAttr::default();
    let mut found = false;

    for attr in attrs {
        if !attr.path().is_ident("i18n") {
            continue;
        }
        found = true;

        // Handle bare #[i18n] — no args, means "use field name as key"
        if matches!(attr.meta, Meta::Path(_)) {
            result.key = None; // will be set to field name later
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                result.key = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("skip") {
                result.skip = true;
                Ok(())
            } else {
                Err(meta.error("unsupported i18n field attribute, expected `key` or `skip`"))
            }
        })?;
    }

    if found {
        Ok(Some(result))
    } else {
        Ok(None)
    }
}
