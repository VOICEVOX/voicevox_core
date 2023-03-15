macro_rules! partialeq {
    (
        for $ty:ty {
            unit: {
                $(
                    $unit_variant_ident:ident
                ),*
                $(,)?
            },
            unnamed: {
                $(
                    $unnamed_variant_ident:ident(
                        $($unnamed_field:ident),*
                        $($(,)? #[stringify] $unnamed_source:ident)?
                        $(,)?
                    )
                ),*
                $(,)?
            },
            named: {
                $(
                    $named_variant_ident:ident {
                        $($named_field:ident),*
                        $($(,)? #[stringify] $named_source:ident)?
                        $(,)?
                    }
                ),*
                $(,)?
            },
        }
    ) => {
        impl PartialEq for $ty {
            fn eq(&self, __other: &Self) -> bool {
                return match (self, __other) {
                    $(
                        (
                            Self::$unit_variant_ident,
                            Self::$unit_variant_ident,
                        ) => true,
                    )*
                    $(
                        (
                            Self::$unnamed_variant_ident($(::paste::paste!([<$unnamed_field _1>]),)* $(::paste::paste!([<$unnamed_source _1>]))*),
                            Self::$unnamed_variant_ident($(::paste::paste!([<$unnamed_field _2>]),)* $(::paste::paste!([<$unnamed_source _2>]))*),
                        ) => true
                            $(
                                && ::paste::paste!([<$unnamed_field _1>])
                                    == ::paste::paste!([<$unnamed_field _2>])
                            )*
                            $(
                                && __OptionalAnyhowError::to_option_string(::paste::paste!([<$unnamed_source _1>]))
                                    == __OptionalAnyhowError::to_option_string(::paste::paste!([<$unnamed_source _2>]))
                            )*,
                    )*
                    $(
                        (
                            Self::$named_variant_ident { $($named_field: ::paste::paste!([<$named_field _1>]),)* $($named_source: ::paste::paste!([<$named_source _1>]))* },
                            Self::$named_variant_ident { $($named_field: ::paste::paste!([<$named_field _2>]),)* $($named_source: ::paste::paste!([<$named_source _2>]))* },
                        ) => true
                            $(
                                && ::paste::paste!([<$named_field _1>])
                                    == ::paste::paste!([<$named_field _2>])
                            )*
                            $(
                                && __OptionalAnyhowError::to_option_string(::paste::paste!([<$named_source _1>]))
                                    == __OptionalAnyhowError::to_option_string(::paste::paste!([<$named_source _2>]))
                            )*,
                    )*
                    _ => false,
                };

                /// バリアントがすべて網羅されていることをチェック
                const _: fn(&$ty) = |err| {
                    type This = $ty;

                    match err {
                        $(| This::$unit_variant_ident)*
                        $(| This::$unnamed_variant_ident(..))*
                        $(| This::$named_variant_ident { .. })* => {}
                    }
                };

                trait __OptionalAnyhowError {
                    fn to_option_string(&self) -> Option<String>;
                }

                impl __OptionalAnyhowError for anyhow::Error {
                    fn to_option_string(&self) -> Option<String> {
                        Some(self.to_string())
                    }
                }

                impl __OptionalAnyhowError for Option<anyhow::Error> {
                    fn to_option_string(&self) -> Option<String> {
                        self.as_ref().map(|e| e.to_string())
                    }
                }
            }
        }
    };
}
pub(crate) use partialeq;
