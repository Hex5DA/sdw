define i64 @main() {
    %x = alloca i64
    %subtemp = sub i64 2, 3
    store i64 %subtemp, ptr %x
    
    %xderef = load i64, ptr %x
    ret i64 %xderef
}
    
