/// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::ast::{Dimension, Statement};

pub const DEFAULT_STRING_LEN: u32 = 80;
#[derive(Debug, PartialEq)]
pub struct DataType {
    pub name: String,
    /// the initial value defined on the TYPE-declration
    pub initial_value: Option<Statement>,
    pub information: DataTypeInformation,
    //TODO : Add location information
}

impl DataType {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_type_information(&self) -> &DataTypeInformation {
        &self.information
    }

    pub fn clone_type_information(&self) -> DataTypeInformation {
        self.information.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataTypeInformation {
    Struct {
        name: String,
        member_names: Vec<String>,
    },
    Array {
        name: String,
        inner_type_name: String,
        dimensions: Vec<Dimension>,
    },
    Integer {
        name: String,
        signed: bool,
        size: u32,
    },
    Float {
        name: String,
        size: u32,
    },
    String {
        size: u32,
    },
    Alias {
        name: String,
        referenced_type: String,
    },
    Void,
}

impl DataTypeInformation {
    pub fn get_name(&self) -> &str {
        match self {
            DataTypeInformation::Struct { name, .. } => name,
            DataTypeInformation::Array { name, .. } => name,
            DataTypeInformation::Integer { name, .. } => name,
            DataTypeInformation::Float { name, .. } => name,
            DataTypeInformation::String { .. } => "String",
            DataTypeInformation::Alias { name, .. } => name,
            DataTypeInformation::Void => "Void",
        }
    }

    pub fn is_int(&self) -> bool {
        if let DataTypeInformation::Integer { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_float(&self) -> bool {
        if let DataTypeInformation::Float { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_numerical(&self) -> bool {
        match self {
            DataTypeInformation::Integer { .. } | DataTypeInformation::Float { .. } => true,
            _ => false,
        }
    }

    pub fn get_size(&self) -> u32 {
        match self {
            DataTypeInformation::Integer { size, .. } => *size,
            DataTypeInformation::Float { size, .. } => *size,
            DataTypeInformation::String { size, .. } => *size,
            DataTypeInformation::Struct { .. } => 0, //TODO : Should we fill in the struct members here for size calculation or save the struct size.
            DataTypeInformation::Array { .. } => unimplemented!(), //Propably length * inner type size
            DataTypeInformation::Alias { .. } => unimplemented!(),
            DataTypeInformation::Void => 0,
        }
    }
}

pub fn get_builtin_types() -> Vec<DataType> {
    let mut res = vec![];
    res.push(DataType {
        name: "__VOID".into(),
        initial_value: None,
        information: DataTypeInformation::Void,
    });
    res.push(DataType {
        name: "BOOL".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "BOOL".into(),
            signed: true,
            size: 1,
        },
    });
    res.push(DataType {
        name: "BYTE".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "BYTE".into(),
            signed: false,
            size: 8,
        },
    });
    res.push(DataType {
        name: "SINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "SINT".into(),
            signed: true,
            size: 8,
        },
    });
    res.push(DataType {
        name: "USINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "USINT".into(),
            signed: false,
            size: 8,
        },
    });
    res.push(DataType {
        name: "WORD".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "WORD".into(),
            signed: false,
            size: 16,
        },
    });
    res.push(DataType {
        name: "INT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "INT".into(),
            signed: true,
            size: 16,
        },
    });
    res.push(DataType {
        name: "UINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "UINT".into(),
            signed: false,
            size: 16,
        },
    });
    res.push(DataType {
        name: "DWORD".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "DWORD".into(),
            signed: false,
            size: 32,
        },
    });
    res.push(DataType {
        name: "DINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "DINT".into(),
            signed: true,
            size: 32,
        },
    });
    res.push(DataType {
        name: "UDINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "UDINT".into(),
            signed: false,
            size: 32,
        },
    });
    res.push(DataType {
        name: "LWORD".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "LWORD".into(),
            signed: false,
            size: 64,
        },
    });
    res.push(DataType {
        name: "LINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "LINT".into(),
            signed: true,
            size: 64,
        },
    });
    res.push(DataType {
        name: "ULINT".into(),
        initial_value: None,
        information: DataTypeInformation::Integer {
            name: "ULINT".into(),
            signed: false,
            size: 64,
        },
    });
    res.push(DataType {
        name: "REAL".into(),
        initial_value: None,
        information: DataTypeInformation::Float {
            name: "REAL".into(),
            size: 32,
        },
    });
    res.push(DataType {
        name: "LREAL".into(),
        initial_value: None,
        information: DataTypeInformation::Float {
            name: "LREAL".into(),
            size: 64,
        },
    });
    res.push(DataType {
        name: "STRING".into(),
        initial_value: None,
        information: DataTypeInformation::String {
            size: DEFAULT_STRING_LEN + 1,
        },
    });
    res
}

pub fn new_string_information<'ctx>(len: u32) -> DataTypeInformation {
    DataTypeInformation::String { size: len + 1 }
}

fn get_rank(type_information: &DataTypeInformation) -> u32 {
    match type_information {
        DataTypeInformation::Integer { signed, size, .. } => {
            if *signed {
                *size + 1
            } else {
                *size
            }
        }
        DataTypeInformation::Float { size, .. } => size + 1000,
        _ => unreachable!(),
    }
}

fn is_same_type_nature(ltype: &DataTypeInformation, rtype: &DataTypeInformation) -> bool {
    (ltype.is_int() && ltype.is_int() == rtype.is_int())
        || (ltype.is_float() && ltype.is_float() == rtype.is_float())
}

fn get_real_type() -> DataTypeInformation {
    DataTypeInformation::Float {
        name: "REAL".into(),
        size: 32,
    }
}

fn get_lreal_type() -> DataTypeInformation {
    DataTypeInformation::Float {
        name: "LREAL".into(),
        size: 64,
    }
}

pub fn get_bigger_type<'a>(
    ltype: &DataTypeInformation,
    rtype: &DataTypeInformation,
) -> DataTypeInformation {
    let bigger_type = if is_same_type_nature(&ltype, &rtype) {
        if get_rank(&ltype) < get_rank(&rtype) {
            rtype.clone()
        } else {
            ltype.clone()
        }
    } else {
        let real_type = get_real_type();
        let real_size = real_type.get_size();
        if ltype.get_size() > real_size || rtype.get_size() > real_size {
            get_lreal_type()
        } else {
            real_type
        }
    };
    bigger_type
}