use std::{alloc::{self, Layout, alloc, dealloc, realloc}, fmt, ops::Add, ptr, slice};

#[derive(PartialEq)]
pub enum ObjType {
    STRING
}

pub struct Obj{
    pub marked: bool,
    pub next: Option<*mut Obj>
}

pub struct StringObject {
    obj: Obj, // obj struct needed for GC
    len: usize, // size of string literal
    ptr: *mut u8 // string literal
}

impl StringObject {
    // create a string obj
    pub fn new(lex: impl Into<String>) -> *mut StringObject {
        unsafe{
            let lexeme = lex.into();
            let len = lexeme.len();
            if len < 1 {
                panic!("Zero-byte allocation: trying to allocate memory less than 1 byte");
            }

            let layout = Layout::from_size_align(len + 1, 1).unwrap_or_else(|error| {
                panic!("{error}");
            });
            let ptr = alloc::alloc(layout); // allocates memory for string array
            if ptr.is_null() {
                panic!("Memory allocation failed for string contents");
            }

            ptr::copy_nonoverlapping(lexeme.as_ptr(), ptr, len as usize); // copy from the lexeme
            // into the allocated memory
            ptr::write(ptr.add(len), 0); //append '\0' to the end

            //create a heap for StringObject
            let strobjlayout = Layout::new::<StringObject>();
            let strobjptr = alloc::alloc(strobjlayout) as *mut StringObject;

            if strobjptr.is_null() {
                dealloc(ptr, layout);
                panic!("Memory allocation failed for StringObject wrapper");
            }

            let obj = Obj{marked: false, next: None};
            let strobj = StringObject {
                ptr: ptr, len: len, obj: obj
            };

            // write the content of the strobj into the memory provided for it
            ptr::write(strobjptr, strobj);
            strobjptr // return the value
        }
    }

    //returns the len of the array of string
    pub fn len(&self) -> usize {
        return self.len;
    }

    pub fn concate(&mut self, another: *mut StringObject) {
        unsafe {
            let nlen = self.len + (*another).len;
            let layout = Layout::from_size_align(self.len + 1, 1).unwrap_or_else(|error| {
                panic!("{error}");
            });

            let nptr = realloc(self.ptr, layout, nlen + 1); //new ptr from 
            if nptr.is_null() {
                panic!("Memory allocation failed for StringObject wrapper");
            }

            ptr::copy_nonoverlapping((*another).ptr, nptr.add(self.len), (*another).len); //append
            //source (content of another.ptr) to destination nptr
            ptr::write(nptr.add(nlen), 0);

            self.ptr = nptr; // assign the new pointer
            self.len = nlen; // assingn len new length
        }
    }

    // reallocate ptr
    pub fn append(&mut self, another: *mut StringObject) {
        unsafe {
            let nlen = self.len + (*another).len;
            let layout = Layout::from_size_align(self.len + 1, 1).unwrap_or_else(|error| {
                panic!("{error}");
            });

            let nptr = realloc(self.ptr, layout, nlen + 1); //new ptr from 
            if nptr.is_null() {
                panic!("Memory allocation failed for StringObject wrapper");
            }

            ptr::copy_nonoverlapping((*another).ptr, nptr.add(self.len), (*another).len); //append
            //source (content of another.ptr) to destination nptr
            ptr::write(nptr.add(nlen), 0);

            self.ptr = nptr; // assign the new pointer
            self.len = nlen; // assingn len new length
        }
    }

    // if the content of String object is the same as String
    pub unsafe fn is_equal_to_string(&self, STR: &str) -> bool {
        /* convert strobj.ptr to [u8]
         * convert STR to [u8], then compare
        */
        let self_slice = slice::from_raw_parts(self.ptr, self.len);
        self_slice == STR.as_bytes()
    }


    pub unsafe fn value(&self) -> &str{
        let string  = unsafe {
            let self_slice = slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8(self_slice).unwrap()
        };
        string
    }

    pub unsafe fn add(&self, rhs: *mut StringObject) -> *mut StringObject {
     unsafe{
        let nlen = self.len + (*rhs).len;
        let layout = Layout::from_size_align(nlen + 1, 1).unwrap(); // create a space wide enought
        // for the 2 string literal
        let nptr = alloc(layout);
        if nptr.is_null() {
                panic!("Memory allocation failed for StringObject wrapper");
        }

        //write the content of the stringobjects to the new pointer
        ptr::copy_nonoverlapping(self.ptr, nptr, self.len); // copy the first stringobject starting
        // from the beginning
        ptr::copy_nonoverlapping((*rhs).ptr, nptr.add(self.len), (*rhs).len); // copy the second
        // starting from where the first ends

        ptr::write(nptr.add(nlen), 0); // null terminate after concating the two stringobjects

        let strobjlayout = Layout::new::<StringObject>();
        let newobjPtr = alloc(strobjlayout) as *mut StringObject;

        let strobj = StringObject {
            obj: Obj { marked: false, next: None },
            len: nlen,
            ptr: nptr
        };

        ptr::write(newobjPtr, strobj);
        newobjPtr
    }
    }
}

impl fmt::Display for StringObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // format *mut u8 to utf
        let string = unsafe {
            str::from_utf8(slice::from_raw_parts(self.ptr, self.len)).unwrap()
        };
        // then write it to f
        write!(f, "{string}")
    }
}

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        // compare the actual bytes/characters
        self.ptr == other.ptr
    }
}

