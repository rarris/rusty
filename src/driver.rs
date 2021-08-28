// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder

/// Compilation options
#[derive(Default)]
pub struct DriverOptions {
    output : &str
}

pub struct Driver {
    files: Vec<&'static str>,
    options: DriverOptions,
}

impl Default for Driver {
    fn default() -> Self {
        Driver {
            files: vec![],
            options: DriverOptions::default(),
        }
    }
}

impl Driver {

    fn from_configration() -> Driver {

    }

    fn add_files(self, files : &[&str]) -> Self{
        self
    }

    fn add_file(self, file : &str) -> Self{
        self
    }

    fn parse(self) -> Self {
        self
    }

    fn annotate(self) -> Self {
        self
    }

    fn index(self) -> Self {
        self
    }

    fn validate(self) -> Self {
        self
    }

    fn codegen(self) -> Self {
        self
    }

    fn link(self) -> Self {
        self
    }
}
