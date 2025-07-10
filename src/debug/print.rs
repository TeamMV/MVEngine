use std::process::Command;
use mvutils::lazy;
use mvutils::print::{Col, Printer};
use mvutils::utils::Time;
use parking_lot::Mutex;
use crate::debug::PROFILER;

lazy! {
    static LAST_PRINT: Mutex<u64> = Mutex::new(0);
}

pub fn print_summary(print_interval_ms: u64) {
    let last = *LAST_PRINT.lock();
    let now = u64::time_millis();
    if now - last < print_interval_ms {
        return;
    }

    {
        let mut lock = LAST_PRINT.lock();
        *lock = now;
    }

    let lock = PROFILER.inner.read();
    let app_draw_time = lock.app_draw.time_nanos();
    let app_update_time = lock.app_update.time_nanos();
    let app_time = app_draw_time + app_update_time;

    let ui_draw_time = lock.ui_draw.time_nanos();
    let ui_compute_time = lock.ui_compute.time_nanos();
    let ui_time = ui_draw_time + ui_compute_time;

    let render_batch_time = lock.render_batch.time_nanos();
    let render_draw_time = lock.render_draw.time_nanos();
    let render_swap_time = lock.render_swap.time_nanos();
    let render_time = render_batch_time + render_draw_time + render_swap_time;

    let ecs_find_time = lock.ecs_find.time_nanos();
    let ecs_time = ecs_find_time;

    let input_time = lock.input.time_nanos();

    let waiting_time = lock.waiting.time_nanos();

    let printer = Printer::start();
    let printer = print_table_header(printer);
    let printer = print_entry(printer, "app", Col::BrightYellow, app_time);
    let printer = print_entry(printer, "app/draw", Col::BrightYellow, app_draw_time);
    let printer = print_entry(printer, "app/update", Col::BrightYellow, app_update_time);
    let printer = print_entry(printer, "ui", Col::Cyan, ui_time);
    let printer = print_entry(printer, "ui/compute", Col::Cyan, ui_compute_time);
    let printer = print_entry(printer, "ui/draw", Col::Cyan, ui_draw_time);
    let printer = print_entry(printer, "render", Col::BrightRed, render_time);
    let printer = print_entry(printer, "render/batch", Col::BrightRed, render_batch_time);
    let printer = print_entry(printer, "render/draw", Col::BrightRed, render_draw_time);
    let printer = print_entry(printer, "render/swap", Col::BrightRed, render_swap_time);
    let printer = print_entry(printer, "ecs", Col::BrightBlue, ecs_time);
    let printer = print_entry(printer, "ecs/find", Col::BrightBlue, ecs_find_time);
    let printer = print_entry(printer, "input", Col::Magenta, input_time);
    let printer = print_entry(printer, "waiting", Col::Grey, waiting_time);

    clear_terminal_screen();
    printer.flush();
}

//derived from https://stackoverflow.com/questions/34837011/how-to-clear-the-terminal-screen-in-rust-after-a-new-line-is-printed
fn clear_terminal_screen() {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).spawn()
    } else {
        // "clear" or "tput reset"
        Command::new("tput").arg("reset").spawn()
    };

    // Alternative solution:
    if result.is_err() {
        print!("{esc}c", esc = 27 as char);
    } else {
        let mut child = result.unwrap();
        let _ = child.wait();
    }
}

fn print_table_header(mut printer: Printer) -> Printer {
    printer
        .col_for(Col::Grey, &format!("{:<12}", "Process"))
        .col_for(Col::Grey, &format!("{:>12}", "Time (ns)"))
        .col_for(Col::Grey, &format!("  {:>11}", "Time (ms)"))
        .ln()
}

fn print_entry(
    mut printer: Printer,
    label: &str,
    label_col: Col,
    time_ns: u64,
) -> Printer {
    let ms = time_ns as f64 / 1_000_000.0;
    printer
        .col_for(label_col, &format!("{label:<12}"))
        .col_for(Col::Lime, &format!("{time_ns:>10}ns"))
        .col_for(Col::Yellow, &format!("  ({ms:>7.3}ms)"))
        .ln()
}