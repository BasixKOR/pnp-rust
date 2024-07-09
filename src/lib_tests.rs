use serde::Deserialize;

use crate::{ResolutionConfig, Resolution, Manifest};

#[derive(Deserialize)]
struct Test {
    it: String,
    imported: String,
    importer: String,
    expected: String,
}

#[derive(Deserialize)]
struct TestSuite {
    manifest: Manifest,
    tests: Vec<Test>,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{init_pnp_manifest, load_pnp_manifest, resolve_to_unqualified, ResolutionHost};
    use super::*;

    #[test]
    fn example() {
        let manifest
            = load_pnp_manifest("data/pnp-yarn-v3.cjs").unwrap();

        let host = ResolutionHost {
            find_pnp_manifest: Box::new(move |_| Ok(Some(manifest.clone()))),
            ..Default::default()
        };

        let config = ResolutionConfig {
            host,
            ..Default::default()
        };

        let resolution = resolve_to_unqualified(
            "lodash/cloneDeep",
            std::path::PathBuf::from("/path/to/file"),
            &config,
        );

        match resolution {
            Ok(Resolution::Resolved(_path, _subpath)) => {
                // path = "/path/to/lodash.zip"
                // subpath = "cloneDeep"
            },
            Ok(Resolution::Skipped) => {
                // This is returned when the PnP resolver decides that it shouldn't
                // handle the resolution for this particular specifier. In that case,
                // the specifier should be forwarded to the default resolver.
            },
            Err(_err) => {
                // An error happened during the resolution. Falling back to the default
                // resolver isn't recommended.
            },
        };
    }

    #[test]
    fn test_load_pnp_manifest() {
        load_pnp_manifest("data/pnp-yarn-v3.cjs")
            .expect("Assertion failed: Expected to load the .pnp.cjs file generated by Yarn 3");

        load_pnp_manifest("data/pnp-yarn-v4.cjs")
            .expect("Assertion failed: Expected to load the .pnp.cjs file generated by Yarn 4");
    }

    #[test]
    fn test_resolve_unqualified() {
        let expectations_path = std::env::current_dir()
            .expect("Assertion failed: Expected a valid current working directory")
            .join("data/test-expectations.json");

        let manifest_content = fs::read_to_string(&expectations_path)
            .expect("Assertion failed: Expected the expectations to be found");

        let mut test_suites: Vec<TestSuite> = serde_json::from_str(&manifest_content)
            .expect("Assertion failed: Expected the expectations to be loaded");

        for test_suite in test_suites.iter_mut() {
            let manifest = &mut test_suite.manifest;
            init_pnp_manifest(manifest, &PathBuf::from("/path/to/project/.pnp.cjs"));

            for test in test_suite.tests.iter() {
                let specifier = &test.imported;
                let parent = &PathBuf::from(&test.importer).join("fooo");

                let manifest_copy = manifest.clone();

                let host = ResolutionHost {
                    find_pnp_manifest: Box::new(move |_| Ok(Some(manifest_copy.clone()))),
                    ..Default::default()
                };

                let config = ResolutionConfig {
                    host,
                    ..Default::default()
                };

                let resolution = resolve_to_unqualified(specifier, parent, &config);

                match resolution {
                    Ok(Resolution::Resolved(path, _subpath)) => {
                        assert_eq!(path.to_string_lossy(), test.expected, "{}", test.it);
                    },
                    Ok(Resolution::Skipped) => {
                        assert_eq!(specifier, &test.expected, "{}", test.it);
                    },
                    Err(err) => {
                        assert_eq!(test.expected, "error!", "{}: {}", test.it, err.to_string());
                    },
                }
                
            }
        }
    }
}
