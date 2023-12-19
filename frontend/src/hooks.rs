use yew::{hook, use_state};
use yew_hooks::use_interval;

#[hook]
pub fn use_retry<Callback, Condition>(callback: Callback, on_retry: Condition, times: u32) -> bool
    where
        Callback: Fn() + 'static,
        Condition: Fn() -> bool + 'static,
{
    let interval = use_state(|| 200);
    let interval_mut = interval.clone();
    let retry = use_state(|| 0);
    use_interval(move || {
        callback();
        if !on_retry() {
            interval_mut.set(0);
            return;
        }
        interval_mut.set(*interval_mut * 2);
        if *retry == times {
            interval_mut.set(0);
        } else {
            retry.set(*retry + 1);
        }
    }, *interval);

    return *interval != 0;
}
