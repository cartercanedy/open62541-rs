use open62541::{DataType as _, ua};

fn main() {
    let elements = &[
        ua::RelativePathElement::init()
            .with_is_inverse(false)
            .with_target_name(&ua::QualifiedName::new(0, "Objects"))
            .with_include_subtypes(true)
            .with_reference_type_id(&ua::NodeId::numeric(0, 33))
    ];

    let rel_path = ua::RelativePath::init()
        .with_elements(elements)
        .with_elements(elements); // should trigger ASAN

     let name_bytes = unsafe {
            let target_ua_str = rel_path
                .into_raw()
                .elements
                .read()
                .targetName
                .name;

            std::slice::from_raw_parts(target_ua_str.data, target_ua_str.length).to_vec()
    };

    println!("{}", String::from_utf8_lossy(&name_bytes));
}
