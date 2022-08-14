use utoipa_corelib::PathOperations2;
use utoipa_gen_actix_web::PathOperations2Ext;

#[test]
fn test_foobar() {
    // TODO

    let c = <PathOperations2 as PathOperations2Ext>::resolve_path(&None::<String>);

    dbg!(&c);
    println!("{c:#?}");
}
