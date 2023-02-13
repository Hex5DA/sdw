define i64 @main() {
    %x = alloca i64
    store i64 10, ptr %x
    %xderef = load i64, ptr %x
    
    %res = icmp sgt i64 %xderef, 50
    br i1 %res, label %truecase, label %falsecase

falsecase:
    ret i64 1 
truecase:
    ret i64 0 
}
    
