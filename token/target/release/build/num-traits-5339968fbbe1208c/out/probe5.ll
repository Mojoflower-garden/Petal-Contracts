; ModuleID = 'probe5.4f676db8d040c0f-cgu.0'
source_filename = "probe5.4f676db8d040c0f-cgu.0"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx11.0.0"

@alloc_49cd4770cae46c688efee7fa9c056542 = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/8ede3aae28fe6e4d52b38157d7bfe0d3bceef225/library/core/src/num/mod.rs" }>, align 1
@alloc_74540e89464df5fd6be485ce28fbfc20 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_49cd4770cae46c688efee7fa9c056542, [16 x i8] c"K\00\00\00\00\00\00\00~\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe5::probe
; Function Attrs: uwtable
define void @_ZN6probe55probe17hc7f46faf83536267E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h8a012096d8ba6d7fE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h3a764673a10f87a9E(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_74540e89464df5fd6be485ce28fbfc20) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h8a012096d8ba6d7fE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17h3a764673a10f87a9E(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn uwtable "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0}

!0 = !{i32 8, !"PIC Level", i32 2}
