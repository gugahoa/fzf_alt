pub mod config {
    use regex::Regex;
    use serde::{Serialize, Deserialize};
    use serde::de::{Deserializer, Visitor, MapAccess};
    use std::collections::HashMap;
    use std::fmt;
    use std::marker::PhantomData;

    #[derive(Serialize)]
    pub struct AppConfig(pub HashMap<String, FiletypeConfig>);

    #[derive(Serialize, Deserialize)]
    pub struct FiletypeConfig {
        #[serde(with = "serde_regex")]
        pub is_test: Regex,
        #[serde(with = "serde_regex")]
        pub strip: Regex,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            let mut config_map = HashMap::new();

            let elixir_config = FiletypeConfig {
                is_test: Regex::new("_test.exs$").unwrap(),
                strip: Regex::new("(?P<p>[^_\\\\/]+)_?(\\w+)?.exs?$").unwrap(),
            };

            config_map.insert("elixir".to_owned(), elixir_config);

            let python_config = FiletypeConfig {
                is_test: Regex::new("(tests|test)_(\\w+).py").unwrap(),
                strip: Regex::new("src/(?P<p>\\w+).py$").unwrap(),
            };

            config_map.insert("python".to_owned(), python_config);

            AppConfig(config_map)
        }
    }


    // A Visitor is a type that holds methods that a Deserializer can drive
    // depending on what is contained in the input data.
    //
    // This is an example of a "zero sized type" in Rust. The PhantomData
    // keeps the compiler from complaining about unused generic type
    // parameters.
    struct AppConfigVisitor {
        marker: PhantomData<fn() -> AppConfig>
    }

    impl AppConfigVisitor {
        fn new() -> Self {
            Self {
                marker: PhantomData
            }
        }
    }

    // This is the trait that Deserializers are going to be driving. There
    // is one method for each type of data that our type knows how to
    // deserialize from. There are many other methods that are not
    // implemented here, for example deserializing from integers or strings.
    // By default those methods will return an error, which makes sense
    // because we cannot deserialize a MyMap from an integer or string.
    impl<'de> Visitor<'de> for AppConfigVisitor
    {
        // The type that our Visitor is going to produce.
        type Value = AppConfig;

        // Format a message stating what data this Visitor expects to receive.
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("fzf_alt app config")
        }

        // Deserialize AppConfig from an abstract "map" provided by the
        // Deserializer. The MapAccess input is a callback provided by
        // the Deserializer to let us see each entry in the map.
        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

            // While there are entries remaining in the input, add them
            // into our map.
            while let Some((key, value)) = access.next_entry()? {
                map.insert(key, value);
            }

            Ok(AppConfig(map))
        }
    }

    impl<'de> Deserialize<'de> for AppConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where 
            D: Deserializer<'de>
        {
            deserializer.deserialize_map(AppConfigVisitor::new())
        }
    }
}
