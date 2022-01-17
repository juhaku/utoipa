
struct Foo<'a> {
    value_type: &'a TypeTuple<'a, ValueType>,
    description: Option<&'a String>,
    generic: Option<&'a GenericType>,
    component_attributes: Option<&'a ComponentAttribute>,
}

impl ToTokens for Foo<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.generic {
            Some(GenericType::Map) => {
                tokens.extend(quote! {
                    utoipa::openapi::Object::new()
                });
                if let Some(ref description) = self.description {
                    tokens.extend(quote! {
                        .with_description(#description)
                    })
                }
            },
            Some(GenericType::Vec) => {
                let f = Foo {
                    description: self.description,
                    generic: None, // TODO this needs some refactoring need support for nested generics
                    value_type: self.value_type,
                    component_attributes: self.component_attributes,
                };
                
                quote! {
                    #f.to_array()
                };
            },
            None => {
                let TypeTuple(value, ident) = self.value_type;
                
                match value {
                    ValueType::Primitive => {
                        let component_type = ComponentType(ident);

                        tokens.extend(quote! {
                            utoipa::openapi::Property::new(
                                #component_type
                            )
                        });

                        if let Some(ref description) = self.description {
                            tokens.extend(quote! {
                                .with_description(#description)
                            })
                        }

                        let format = ComponentFormat(ident);
                        if format.is_known_format() {
                            tokens.extend(quote! {
                                .with_format(#format)
                            })
                        }
                    }
                    ValueType::Object => {
                        let object_name = &*ident.to_string();

                        tokens.extend(quote! {
                            utoipa::openapi::Ref::from_component_name(#object_name)
                        })
                    }
                }
            },
            _ => unreachable!("Some(GenericType::Option) is not valid scneario optionality is checked otherwise!!!")
        }

        if let Some(component_attribute) = self.component_attributes {
            component_attribute
                .0
                .iter()
                .map(|attribute_type| match attribute_type {
                    AttributeType::Default(..) => quote! {
                        .with_default(#attribute_type)
                    },
                    AttributeType::Example(..) => quote! {
                        .with_example(#attribute_type)
                    },
                    AttributeType::Format(..) => quote! {
                        .with_format(#attribute_type)
                    },
                })
                .for_each(|stream| tokens.extend(stream))
        }
    }
}