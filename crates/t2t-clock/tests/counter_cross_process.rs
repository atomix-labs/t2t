//! Two processes read one counter: a reading a child process takes lies between two its parent
//! takes around the child's whole life, which no per-process counter would.

#[cfg(test)]
#[cfg(counter)]
mod tests {
    use std::env;
    use std::process::Command;

    use t2t_clock::{Clock, Counter};
    use t2t_core::{Tick, Timedelta};

    /// The variable whose presence makes the test print one reading and stop, as the child.
    const CHILD: &str = "T2T_COUNTER_CHILD";

    #[test]
    #[cfg_attr(miri, ignore = "Miri runs no child process")]
    fn a_reading_one_process_takes_another_may_subtract_from() {
        let counter = Counter::discover().expect("a counter with a rate");
        if env::var_os(CHILD).is_some() {
            println!("{}", counter.now());
            return;
        }

        let start = counter.now();
        let output = Command::new(env::current_exe().expect("the test binary"))
            .args(["--exact", "tests::a_reading_one_process_takes_another_may_subtract_from"])
            .arg("--nocapture")
            .env(CHILD, "1")
            .output()
            .expect("the child runs");
        let end = counter.now();

        assert!(output.status.success(), "the child succeeded: {output:?}");
        let child_reading: Tick = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.trim().parse().ok())
            .expect("the child printed a reading");
        assert!(start < child_reading, "the child read after the parent began: {start}");
        assert!(child_reading < end, "and before the parent ended: {end}");
        let took = (end - start).to_timedelta(counter.rate());
        assert!(took < Timedelta::from_secs(30), "a quick child: {took}");
    }
}
