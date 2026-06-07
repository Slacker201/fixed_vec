use crate::fixed_vec::FixedVec;

mod fixed_vec;



#[test]
fn q() {
    let mut fv = FixedVec::new(69);
    fv.push(());
    fv.push(());
    for item in fv {
        println!("1")
    }
}