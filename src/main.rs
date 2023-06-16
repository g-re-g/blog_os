// in src/main.rs

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Whatup big {}", "Dog");

    // invoke a breakpoint exception
    // x86_64::instructions::interrupts::int3(); // new

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };

    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // trigger a stack overflow
    // stack_overflow();
    // let ptr = 0x204e2b as *mut u8;

    // // read from a code page
    // unsafe {
    //     let x = *ptr;
    // }
    // println!("read worked");

    // // write to a code page
    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked");

    // use x86_64::registers::control::Cr3;

    // let (level_4_page_table, _) = Cr3::read();
    // println!(
    //     "Level 4 page table at: {:?}",
    //     level_4_page_table.start_address()
    // );

    // use blog_os::memory::active_level_4_table;
    use x86_64::VirtAddr;
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);
    //     }
    //     // get the physical address from the entry and convert it
    //     let phys = entry.frame().unwrap().start_address();
    //     let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //     let ptr = VirtAddr::new(virt).as_mut_ptr();
    //     let l3_table: &PageTable = unsafe { &*ptr };

    //     // print non-empty entries of the level 3 table
    //     for (i, entry) in l3_table.iter().enumerate() {
    //         if !entry.is_unused() {
    //             println!("  L3 Entry {}: {:?}", i, entry);
    //         }
    //     }
    // }

    // use blog_os::memory::translate_addr;
    // new: different imports
    // use blog_os::memory;
    // use x86_64::structures::paging::Translate;
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // // println!("PHYS MEM OFFSET {:?}", phys_mem_offset);

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     // let virt = VirtAddr::new(address);
    //     // let phys = unsafe { translate_addr(virt, phys_mem_offset) };
    //     // println!("{:?} -> {:?}", virt, phys);
    //     let virt = VirtAddr::new(address);
    //     // new: use the `mapper.translate_addr` method
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // use blog_os::memory::BootInfoFrameAllocator;

    // // map an unused page
    // let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // let page = Page::containing_address(VirtAddr::new(0x0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    extern crate alloc;
    use alloc::boxed::Box;

    // let _x = Box::new(41);

    use blog_os::allocator; // new import
    use blog_os::memory::{self, BootInfoFrameAllocator};

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    drop(heap_value);
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    use blog_os::task::executor::Executor;
    use blog_os::task::{keyboard, Task}; // new

    let mut executor = Executor::new(); // new

    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
