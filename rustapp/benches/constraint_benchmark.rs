use criterion::Criterion;

use rustapp::prob_manager::constraint::{CollectiveConstraint, GroupConstraint};
use rustapp::history_public::Card;
use criterion::{black_box, criterion_group, criterion_main};
criterion_group!(benches, benchmark_checks);
criterion_main!(benches);
fn benchmark_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("constraint_checks");
    
    // Configure the group if needed
    group.sample_size(250);
    group.warm_up_time(std::time::Duration::from_secs(1));
    let mut store: Vec<(CollectiveConstraint, usize, Card)> = Vec::with_capacity(25);
    store.push(test0());
    store.push(test1());
    store.push(test2());
    store.push(test3());
    store.push(test4());
    store.push(test5());
    store.push(test6());
    store.push(test7());
    store.push(test8());
    store.push(test9());
    store.push(test10());
    store.push(test11());
    store.push(test12());
    store.push(test13());
    store.push(test14());
    store.push(test15());
    store.push(test16());
    store.push(test17());
    store.push(test18());
    store.push(test19());
    store.push(test20());
    store.push(test21());
    group.bench_function("21 Constraint Check", |b| {
        b.iter(|| {
            for (a, b, c) in &store {
                black_box(CollectiveConstraint::player_can_have_active_card_pub(a, *b, c));
            }
        })
    });

    group.finish();
}






















fn test0() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();
    colcon.add_public_constraint(5, Card::Assassin);

    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(3, Card::Contessa);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(2, Card::Duke);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Contessa, 0,  1);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 1, 1], Card::Captain, 0, 2 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 1 );
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke , 0, 1 );
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Captain , 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);
    log::info!(" === Test 7 === ");
    // colcon.printlog();

    return (colcon, 1, Card::Assassin)
    // if output {
    //     println!("Test 7 Legal Correct");
    // } else {
    //     println!("Test 7 Illegal Wrong");
    // }
}
fn test1() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(0, Card::Assassin);

    colcon.add_public_constraint(4, Card::Captain);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(1, Card::Captain);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0,  2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Captain, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Ambassador, 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    // log::info!(" === Test 8 === ");
    // colcon.printlog();
    let dead_hm = colcon.dead_card_count();
    // log::trace!("dead_card_count: {:?}", dead_hm);
    return (colcon, 2, Card::Ambassador)

    // if output {
    //     println!("Test 8 Legal Wrong");
    // } else {
    //     println!("Test 8 Illegal Correct");
    // }
}
fn test2() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Captain);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(3, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Captain, 0,  2);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 1, 0, 1], Card::Contessa, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Assassin, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Duke, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);

    // log::info!(" === Test 9 === ");
    // colcon.printlog();

    return (colcon, 5, Card::Contessa)
    //     println!("Test 9 Legal Correct");
    // } else {
    //     println!("Test 9 Illegal Wrong");
    // }
}
fn test3() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(3, Card::Duke);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(0, Card::Contessa);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Duke, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Duke, 0, 1 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);


    log::info!(" === Test 10 === ");
    // colcon.printlog();

    return (colcon, 1, Card::Assassin)
    // if output {
    //     println!("Test 10 Legal Correct");
    // } else {
    //     println!("Test 10 Illegal Wrong");
    // }
}
fn test4() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(5, Card::Duke);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);

    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 1, 1, 0, 1], Card::Captain, 0, 3);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Duke, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 1, 0, 1], Card::Captain, 0, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Ambassador, 0, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Assassin, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);


    log::info!(" === Test 11 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Ambassador)
    // if output {
    //     println!("Test 11 Legal Wrong");
    // } else {
    //     println!("Test 11 Illegal Correct");
    // }
}
fn test5() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Captain);
    colcon.add_public_constraint(1, Card::Duke);
    colcon.add_public_constraint(5, Card::Captain);
    colcon.add_public_constraint(5, Card::Duke);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Contessa);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Ambassador, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 1, 0, 1], Card::Captain, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Ambassador, 0, 3);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Assassin, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);


    log::info!(" === Test 12 === ");
    // colcon.printlog();

    return (colcon, 2, Card::Assassin)
    // if output {
    //     println!("Test 12 Legal Wrong");
    // } else {
    //     println!("Test 12 Illegal Correct");
    // }
}
fn test6() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(5, Card::Captain);
    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(4, Card::Captain);
    colcon.add_public_constraint(4, Card::Assassin);
    colcon.add_public_constraint(0, Card::Assassin);
    colcon.add_public_constraint(0, Card::Contessa);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Captain);
    colcon.add_public_constraint(2, Card::Assassin);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Ambassador, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Contessa, 1, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);


    log::info!(" === Test 13 === ");
    // colcon.printlog();

    return (colcon, 1, Card::Duke)
    //     println!("Test 13 Legal Correct");
    // } else {
    //     println!("Test 13 Illegal Wrong");
    // }
}
fn test7() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(5, Card::Captain);

    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(4, Card::Assassin);
    colcon.add_public_constraint(4, Card::Assassin);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 3 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Assassin, 0, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Contessa, 0, 2);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);

    log::info!(" === Test 14 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Contessa)
    //     println!("Test 14 Legal Correct");
    // } else {
    //     println!("Test 14 Illegal Wrong");
    // }
}
fn test8() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(1, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(3, Card::Assassin);

    colcon.add_public_constraint(0, Card::Assassin);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(4, Card::Assassin);
    colcon.add_public_constraint(4, Card::Captain);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Captain, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Duke, 0, 1 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Ambassador, 0, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Contessa, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);

    log::info!(" === Test 15 === ");
    // colcon.printlog();

    return (colcon, 2, Card::Duke)
    // if output {
    //     println!("Test 15 Legal Wrong");
    // } else {
    //     println!("Test 15 Illegal Correct");
    // }
}
fn test9() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(0, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Captain);
    colcon.add_public_constraint(2, Card::Assassin);

    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(1, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 0, 1], Card::Duke, 0, 1);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Captain, 0, 2 );
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);

    log::info!(" === Test 16 === ");
    // colcon.printlog();

    return (colcon, 0, Card::Duke)
    // if output {
    //     println!("Test 16 Legal Wrong");
    // } else {
    //     println!("Test 16 Illegal Correct");
    // }
}
fn test10() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Duke);
    colcon.add_public_constraint(3, Card::Contessa);

    colcon.add_public_constraint(0, Card::Assassin);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(1, Card::Duke);
    colcon.add_public_constraint(1, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Assassin);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 3);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 2 );
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);

    log::info!(" === Test 17 === ");
    // colcon.printlog();

    return (colcon, 2, Card::Ambassador)
    // if output {
    //     println!("Test 17 Legal Wrong");
    // } else {
    //     println!("Test 17 Illegal Correct");
    // }
}
fn test11() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();


    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(4, Card::Duke);
    colcon.add_public_constraint(5, Card::Captain);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(0, Card::Assassin);

    colcon.add_public_constraint(1, Card::Duke);
    colcon.add_public_constraint(1, Card::Ambassador);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Ambassador, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Duke, 0, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Assassin, 1, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Contessa, 1, 1);
    let group6: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);
    colcon.add_raw_group(group6);

    log::info!(" === Test 18 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Contessa)
    // if output {
    //     println!("Test 18 Legal Wrong");
    // } else {
    //     println!("Test 18 Illegal Correct");
    // }
}
fn test12() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(5, Card::Contessa);

    colcon.add_public_constraint(2, Card::Captain);
    colcon.add_public_constraint(2, Card::Duke);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(1, Card::Captain);
    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(4, Card::Captain);
    colcon.add_public_constraint(4, Card::Assassin);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Contessa, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Assassin, 1, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    // log::info!(" === Test 19 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Ambassador)
    //     println!("Test 19 Legal Correct");
    // } else {
    //     println!("Test 19 Illegal Wrong");
    // }
}
fn test13() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(2, Card::Captain);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(4, Card::Assassin);

    colcon.add_public_constraint(0, Card::Assassin);
    colcon.add_public_constraint(0, Card::Duke);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Ambassador, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 1, 0, 0, 0, 1], Card::Contessa, 1, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Captain, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);

    // log::info!(" === Test 20 === ");
    // colcon.printlog();

    return (colcon, 2, Card::Captain)
    // if output {
    //     println!("Test 20 Legal Wrong");
    // } else {
    //     println!("Test 20 Illegal Correct");
    // }
}
fn test14() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(5, Card::Ambassador);
    
    colcon.add_public_constraint(0, Card::Ambassador);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(1, Card::Captain);
    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(4, Card::Ambassador);
    colcon.add_public_constraint(4, Card::Duke);
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Duke, 0, 1);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 2);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Assassin, 0, 2);
        
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    // log::info!(" === Test 21 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Assassin)
    //     println!("Test 21 Legal Correct");
    // } else {
    //     println!("Test 21 Illegal Wrong");
    // }
}
fn test15() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Ambassador);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(3, Card::Duke);

    colcon.add_public_constraint(5, Card::Contessa);
    colcon.add_public_constraint(5, Card::Assassin);

    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(4, Card::Duke);
    
    let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Duke, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Captain, 0, 2);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 1, 0, 0, 1], Card::Contessa, 0, 1);
    let group6: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);
    colcon.add_raw_group(group6);

    log::info!(" === Test 22 === ");
    // colcon.printlog();

    return (colcon, 0, Card::Assassin)
    // if output {
    //     println!("Test 22 Legal Wrong");
    // } else {
    //     println!("Test 22 Illegal Correct");
    // }
}
fn test16() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(5, Card::Contessa);

    colcon.add_public_constraint(0, Card::Ambassador);
    colcon.add_public_constraint(0, Card::Assassin);
    colcon.add_public_constraint(1, Card::Ambassador);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(2, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Assassin);

    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 0, 1], Card::Captain, 0, 3);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Duke, 0, 2);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);

    // log::info!(" === Test 23 === ");
    // colcon.printlog();

    return (colcon, 3, Card::Duke)
    //     println!("Test 23 Legal Correct");
    // } else {
    //     println!("Test 23 Illegal Wrong");
    // }
}
fn test17() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(0, Card::Ambassador);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Duke);
    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(5, Card::Assassin);
    colcon.add_public_constraint(5, Card::Duke);

    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 1, 0, 1], Card::Captain, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Assassin, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Contessa, 1, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    // log::info!(" === Test 24 === ");
    // colcon.printlog();

    return (colcon, 1, Card::Captain)
    //     println!("Test 24 Legal Correct");
    // } else {
    //     println!("Test 24 Illegal Wrong");
    // }
}
fn test18() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(0, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Contessa);
    colcon.add_public_constraint(5, Card::Duke);

    colcon.add_public_constraint(1, Card::Contessa);
    colcon.add_public_constraint(1, Card::Assassin);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Contessa);

    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Duke, 1, 1);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 1, 1], Card::Assassin, 0, 2);
    let group3: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Ambassador, 1, 1);
    let group4: GroupConstraint = GroupConstraint::new_list([1, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 2);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Captain, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);

    // log::info!(" === Test 25 === ");
    // colcon.printlog();

    return (colcon, 5, Card::Captain)
    // if output {
    //     println!("Test 25 Legal Wrong");
    // } else {
    //     println!("Test 25 Illegal Correct");
    // }
}
fn test19 () -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(5, Card::Contessa);

    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(4, Card::Ambassador);
    colcon.add_public_constraint(4, Card::Duke);
    colcon.add_public_constraint(3, Card::Ambassador);
    colcon.add_public_constraint(3, Card::Assassin);
    colcon.add_public_constraint(1, Card::Ambassador);
    colcon.add_public_constraint(1, Card::Contessa);

    
    let group1: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Contessa, 0, 1);
    let group2: GroupConstraint = GroupConstraint::new_list([1, 0, 0, 0, 0, 0, 1], Card::Duke, 1, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Captain, 0, 2);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);

    // log::info!(" === Test 26 === ");
    // colcon.printlog();

    return (colcon, 0, Card::Captain)
    //     println!("Test 26 Legal Correct");
    // } else {
    //     println!("Test 26 Illegal Wrong");
    // }
}
fn test20() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(2, Card::Ambassador);
    colcon.add_public_constraint(5, Card::Captain);

    colcon.add_public_constraint(1, Card::Captain);
    colcon.add_public_constraint(1, Card::Duke);
    colcon.add_public_constraint(0, Card::Captain);
    colcon.add_public_constraint(0, Card::Duke);
    colcon.add_public_constraint(4, Card::Contessa);
    colcon.add_public_constraint(4, Card::Duke);
    
    
    let group1: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 1, 0, 1, 1], Card::Ambassador, 0, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Ambassador, 0, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 0, 1, 0, 0, 0, 1], Card::Contessa, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 0, 0, 0, 0, 1, 1], Card::Assassin, 0, 2);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);

    // log::info!(" === Test 27 === ");
    // colcon.printlog();

    return (colcon, 2, Card::Assassin)
    // if output {
    //     println!("Test 27 Legal Wrong");
    // } else {
    //     println!("Test 27 Illegal Correct");
    // }
}
fn test21() -> (CollectiveConstraint, usize, Card) {
    let mut colcon = CollectiveConstraint::new();

    colcon.add_public_constraint(4, Card::Ambassador);
    colcon.add_public_constraint(2, Card::Assassin);
    colcon.add_public_constraint(5, Card::Captain);

    
    
    let group1: GroupConstraint = GroupConstraint::new_list([1, 1, 0, 0, 1, 0, 1], Card::Ambassador, 1, 2);
    let group2: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Ambassador, 1, 1);
    let group3: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Duke, 0, 2);
    let group4: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 1, 0, 1], Card::Assassin, 0, 1);
    let group5: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 1, 1], Card::Contessa, 0, 2);
    let group6: GroupConstraint = GroupConstraint::new_list([0, 1, 0, 0, 0, 0, 1], Card::Captain, 0, 1);
    
    colcon.add_raw_group(group1);
    colcon.add_raw_group(group2);
    colcon.add_raw_group(group3);
    colcon.add_raw_group(group4);
    colcon.add_raw_group(group5);
    colcon.add_raw_group(group6);

    // log::info!(" === Test 28 === ");
    // colcon.printlog();

    return (colcon, 4, Card::Contessa)
    // if output {
    //     println!("Test 28 Inferred Group needed Legal Wrong");
    // } else {
    //     println!("Test 28 Inferred Group needed Illegal Correct");
    // }
}