@start inc eax
push eax
jnz @start
jnz @cont
@loop push ebx
jmp @loop
db 0x0f, 0x04
@cont ret
add eax,1
