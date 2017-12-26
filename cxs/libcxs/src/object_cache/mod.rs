use std::sync::Mutex;
use std::sync::MutexGuard;
use std::collections::HashMap;
use utils::error2::{CxsResult, Error2};
use std::ops::Deref;
use std::ops::DerefMut;

struct ObjectCache<T>{
    store: HashMap<u32, Mutex<T>>,
}

impl<T> Default for ObjectCache<T> {
    fn default() -> ObjectCache<T>
    {
        let store : HashMap<u32, Mutex<T>> = Default::default();
        ObjectCache {
            store
        }
    }
}

impl<T> ObjectCache<T> {
    fn lock_obj(&self, handle:u32) -> Result<MutexGuard<T>,u32>
    {
        match self.store.get(&handle) {
            Some(m) => match m.lock() {
                Ok(obj) => Ok(obj),
                Err(err) => return Err(10) //TODO better error
            },
            None => return Err(10) //TODO Handle not found error
        }
    }

    fn lock_mut_obj(&mut self, handle:u32) -> Result<MutexGuard<T>,u32>
    {
        match self.store.get_mut(&handle) {
            Some(m) => match m.lock() {
                Ok(obj) => Ok(obj),
                Err(err) => return Err(10) //TODO better error
            },
            None => return Err(10) //TODO Handle not found error
        }
    }

    fn get<F,R>(&self, handle:u32, closure: F) -> Result<R,u32>
        where F: Fn(&T) -> Result<R,u32> {
        let obj = self.lock_obj(handle)?;
        closure(obj.deref())
    }

    fn get_mut<F, R>(&mut self, handle:u32, closure: F) -> Result<R,u32>
        where F: Fn(&mut T) -> Result<R,u32> {
        let mut obj = self.lock_mut_obj(handle)?;
        closure(obj.deref_mut())
    }

    fn add(&mut self, handle:u32, obj:T) -> CxsResult<u32> {
        match self.store.insert(handle, Mutex::new(obj)){
            Some(old_obj) => Ok(0),
            None => Ok(Error2::SUCCESS.code_num())
        }
    }

    fn release(&mut self, handle:u32) -> Result<u32,u32> {
//        let mut map = self.lock_map()?;
        match self.store.remove(&handle){
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
        let mut test:ObjectCache<u32> = Default::default();
        let temp = test.add(234, 2222).unwrap();
        let rtn = test.get(234, |obj| Ok(obj.clone()));
        assert_eq!(2222, rtn.unwrap())
    }


    #[test]
    fn sdf_to_string_test() {
        let mut test:ObjectCache<u32> = Default::default();
        let temp = test.add(234, 2222).unwrap();
        let string: String = test.get(234, |obj|{
           Ok(String::from("TEST"))
        }).unwrap();

        assert_eq!("TEST", string);

    }

    fn mut_object_test(){
        let mut test:ObjectCache<String> = Default::default();
        let temp = test.add(234, String::from("TEST")).unwrap();

        test.get_mut(234, |obj|{
            obj.to_lowercase();
            Ok(())
        }).unwrap();

        let string: String = test.get(234, |obj|{
            Ok(obj.clone())
        }).unwrap();

        assert_eq!("test", string);
    }

}
