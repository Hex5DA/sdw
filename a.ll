define i1 @main() {
  ; allocating 'a'
  %a.0 = alloca i1
  %_ct.0 = icmp sgt i1 8, 4
  store i1 %_ct.0, ptr %a.0
  ; dereferencing 'a'
  %a.1 = load i1, ptr %a.0
  ret i1 %a.1
}

