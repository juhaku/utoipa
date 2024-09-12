fn main() {
    utoipa_config::Config::new()
        .alias_for("MyType", "bool")
        .alias_for("MyInt", "Option<i32>")
        .alias_for("MyValue", "str")
        .alias_for("MyDateTime", "String")
        .write_to_file()
}
