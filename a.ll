define i64 @second(i64 %a) {
  %.1 = alloca i64
  store i64 %a, ptr %.1
  ret i64 1
  unreachable
}
define i64 @main() {
  ; allocating 'a'
  %.2 = alloca i64
  store i64 5, ptr %.2
  store i64 10, ptr %.2
  ; dereferencing 'a'
  %.3 = load i64, ptr %.2
  call i64 @second(i64 %.3)
  ; dereferencing 'a'
  %.4 = load i64, ptr %.2
  ret i64 %.4
  unreachable
}

