//! Controls of real pinned storage hooks; not acceptance of the enclosing budgeted owner.
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

#[test]
fn zero_constructor_and_capacity_before_geometry() {
    let mut terminal = Terminal::r3_empty_fixed(TestBackend::new(120, 40)).unwrap();
    let empty = terminal.r3_buffer_storage().unwrap();
    assert_eq!(empty.iter().map(|s| s.cell_capacity).sum::<usize>(), 0);
    assert!(
        !terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 120, 40))
            .unwrap()
    );
    assert_eq!(terminal.r3_buffer_storage().unwrap(), empty);
    terminal.r3_reserve_buffer(false, 4800).unwrap();
    assert!(
        !terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 120, 40))
            .unwrap()
    );
    assert_eq!(terminal.r3_buffer_storage().unwrap()[0].cell_capacity, 4800);
    assert_eq!(terminal.r3_buffer_storage().unwrap()[1].cell_capacity, 0);
    terminal.r3_reserve_buffer(true, 4800).unwrap();
    assert!(
        terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 120, 40))
            .unwrap()
    );
    assert_eq!(terminal.get_frame().area(), Rect::new(0, 0, 120, 40));
}

#[test]
fn fixed_draw_does_not_follow_backend_growth_and_shrink_retains_capacity() {
    let mut terminal = Terminal::r3_empty_fixed(TestBackend::new(120, 40)).unwrap();
    terminal.r3_reserve_buffer(false, 4800).unwrap();
    terminal.r3_reserve_buffer(true, 4800).unwrap();
    assert!(
        terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 120, 40))
            .unwrap()
    );
    let before = terminal.r3_buffer_storage().unwrap();
    terminal.backend_mut().resize(121, 40);
    terminal
        .draw(|frame| assert_eq!(frame.area(), Rect::new(0, 0, 120, 40)))
        .unwrap();
    assert_eq!(terminal.r3_buffer_storage().unwrap(), before);
    assert!(
        !terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 121, 40))
            .unwrap()
    );
    terminal.r3_reserve_buffer(false, 4840).unwrap();
    terminal.r3_reserve_buffer(true, 4840).unwrap();
    assert!(
        terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 121, 40))
            .unwrap()
    );
    let large = terminal.r3_buffer_storage().unwrap();
    assert!(
        terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 80, 24))
            .unwrap()
    );
    assert_eq!(terminal.r3_buffer_storage().unwrap(), large);
}

#[test]
fn readback_counts_real_symbol_heap_without_copy() {
    let mut terminal = Terminal::r3_empty_fixed(TestBackend::new(3, 1)).unwrap();
    terminal.r3_reserve_buffer(false, 3).unwrap();
    terminal.r3_reserve_buffer(true, 3).unwrap();
    assert!(
        terminal
            .r3_resize_reserved_fixed(Rect::new(0, 0, 3, 1))
            .unwrap()
    );
    let symbol = "x".repeat(33);
    terminal
        .draw(|frame| {
            frame.buffer_mut()[(0, 0)].set_symbol(&symbol);
        })
        .unwrap();
    let receipt = terminal.r3_buffer_storage().unwrap();
    assert_eq!(
        receipt.iter().map(|s| s.symbol_heap_bytes).sum::<usize>(),
        33
    );
    assert_eq!(receipt.iter().map(|s| s.cell_capacity).sum::<usize>(), 6);
    terminal.draw(|_| {}).unwrap();
    assert_eq!(
        terminal
            .r3_buffer_storage()
            .unwrap()
            .iter()
            .map(|s| s.symbol_heap_bytes)
            .sum::<usize>(),
        0
    );
}
