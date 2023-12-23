fn main() {
    let tasks = srp::task_sets::task_set1();

    println!("Task set\n{}", tasks);

    // println!("tasks {:?}", &tasks);
    println!("tot_util {}", tasks.total_utilization());

    // let (ip, tr) = pre_analysis(&tasks);
    // println!("ip: {:?}", ip);
    // println!("tr: {:?}", tr);
}
