fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../windows/baddel.ico");
        res.set("ProductName", "Baddel");
        res.set("FileDescription", "Baddel - fix wrong keyboard language");
        res.set("CompanyName", "Ibrahim");
        res.set("LegalCopyright", "MIT License");
        let _ = res.compile();
    }
}
