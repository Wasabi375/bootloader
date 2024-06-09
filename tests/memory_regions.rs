use std::mem::MaybeUninit;

use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use bootloader_x86_64_common::legacy_memory_region::{LegacyFrameAllocator, LegacyMemoryRegion};
use x86_64::{structures::paging::FrameAllocator, PhysAddr};

#[derive(Copy, Clone, Debug)]
struct TestMemoryRegion {
    start: PhysAddr,
    len: u64,
    kind: MemoryRegionKind,
}

impl LegacyMemoryRegion for TestMemoryRegion {
    fn start(&self) -> PhysAddr {
        self.start
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn kind(&self) -> MemoryRegionKind {
        self.kind
    }

    fn usable_after_bootloader_exit(&self) -> bool {
        match self.kind {
            MemoryRegionKind::Usable => true,
            _ => false,
        }
    }
}

const MAX_PHYS_ADDR: u64 = 0x7FFFFF;
fn create_single_test_region() -> Vec<TestMemoryRegion> {
    use self::MemoryRegionKind as Kind;
    use self::TestMemoryRegion as Region;
    vec![Region {
        start: PhysAddr::new(0),
        len: MAX_PHYS_ADDR,
        kind: Kind::Usable,
    }]
}

#[test]
fn test_kernel_and_ram_in_same_region() {
    let regions = create_single_test_region();
    let mut allocator = LegacyFrameAllocator::new(regions.into_iter());

    let mut regions = [MaybeUninit::uninit(); 10];
    let kernel_slice_start = PhysAddr::new(0x50000);
    let kernel_slice_len = 0x1000;
    let ramdisk_slice_start = Some(PhysAddr::new(0x60000));
    let ramdisk_slice_len = 0x2000;

    let kernel_regions = allocator.construct_memory_map(
        &mut regions,
        kernel_slice_start,
        kernel_slice_len,
        ramdisk_slice_start,
        ramdisk_slice_len,
    );
    let mut kernel_regions = kernel_regions.iter();
    // first frame is allways "bootloader" rust does not support null pointers
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0,
            end: 0x1000,
            kind: MemoryRegionKind::Bootloader
        })
    );
    // usable memory before the kernel
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0x1000,
            end: 0x50000,
            kind: MemoryRegionKind::Usable
        })
    );
    // kernel
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0x50000,
            end: 0x51000,
            kind: MemoryRegionKind::Bootloader
        })
    );
    // usabel memory between kernel and ramdisk
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0x51000,
            end: 0x60000,
            kind: MemoryRegionKind::Usable
        })
    );
    // ramdisk
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0x60000,
            end: 0x61000,
            kind: MemoryRegionKind::Bootloader
        })
    );
    // usabele memory after ramdisk
    assert_eq!(
        kernel_regions.next(),
        Some(&MemoryRegion {
            start: 0x61000,
            end: MAX_PHYS_ADDR,
            kind: MemoryRegionKind::Usable
        })
    );
    assert_eq!(kernel_regions.next(), None);
}
