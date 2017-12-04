use std::sync::Mutex;
use std::sync::MutexGuard;
use std::collections::HashMap;
use utils::error2::{CxsResult, Error2};

struct ObjectCache<T>{
    store: Mutex<HashMap<u32, T>>,
}

impl<T> Default for ObjectCache<T> {
    fn default() -> ObjectCache<T>
    {
        let store : Mutex<HashMap<u32, T>> = Default::default();
        ObjectCache {
            store
        }
    }
}

impl<T> ObjectCache<T> {
    fn lock_map(&self) -> Result<MutexGuard<HashMap<u32, T>>,u32>
    {
        match self.store.lock() {
            Ok(map) => Ok(map),
            Err(err) => return Err(10) //TODO better error
        }
    }

    fn get<F>(&self, handle:u32, closure: F) -> Result<u32,u32>
        where F: Fn(&T) -> Result<u32,u32> {

        let map = self.lock_map()?;
        match map.get(&handle) {
            Some(obj) => closure(&obj),
            None => return Err(10) //TODO better error
        }
    }

    fn get_mod<F>(&self, handle:u32, closure: F) -> Result<u32,u32>
        where F: Fn(&T) -> Result<u32,u32> {

        let mut map = self.lock_map()?;

        match map.get_mut(&handle) {
            Some(mut obj) => closure(&mut obj),
            None => return Err(10) //TODO better error
        }
    }

    fn add(&self, handle:u32, obj:T) -> CxsResult<u32> {
        let mut map = self.lock_map().unwrap(); //TODO no unwrap
        match map.insert(handle, obj){
            Some(old_obj) => Ok(0),
            None => Ok(Error2::SUCCESS.code_num())
        }
    }

    fn release(&self, handle:u32) -> Result<u32,u32> {
        let mut map = self.lock_map()?;
        match map.remove(&handle){
            Some(obj) => Ok(0),
            None => Err(10) //TODO better error
        }
    }
}

#[cfg(test)]
mod tests{
    use object_cache::ObjectCache;

    #[test]
    fn sdf_create_test(){
        let c:ObjectCache<u32> = Default::default();
    }

    #[test]
    fn sdf_get_closure(){
        let test:ObjectCache<u32> = Default::default();
        let temp = test.add(234, 2222).unwrap();
        let rtn = test.get(234, |obj| Ok(obj.clone()));
        assert_eq!(2222, rtn.unwrap())
    }
}
