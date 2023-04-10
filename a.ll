define i64 @main() {
  ; allocating 'a'
  %1 = alloca i64
  ; 'addition' binop
  %2 = add i64 4, 8
  store i64 %2, ptr %1
  ; if
  ; dereferencing 'a'
  %3 = load i64, ptr %1
  %4 = icmp sgt i64 %3, 15
  br i1 %4, label %5, label %6
5:
  ret i64 1
6:
  ret i64 0
}

