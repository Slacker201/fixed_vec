# What is this?
This is a fixed vector implementation that can borrow or own memory.

# Why would you use this?
1. It allows runtime determined capacity
2. You can provide a pointer and length, or a mutable slice reference
3. You can allocate your own memory, or convert a vec or box into a `FixedVec`.
4. Zero cost ownership semantics