use std::any::Any;
trait  IAnyData : Any + 'static  {
    fn as_any(&self) -> &dyn Any; 
}

impl IAnyData  for Rectangle {
    fn as_any(&self) -> &dyn Any { self }  
}
impl IAnyData  for Circle  {
    fn as_any(&self) -> &dyn Any { self }
}

struct Rectangle {
    width : u32,
    height : u32,
} 
struct Circle {
    x : u32,
    y : u32,
    radius : u32,
}

pub fn test(){
    let rect = Box::new( Rectangle { width: 4, height: 6});
let circle = Box::new( Circle { x: 0, y:0, radius: 5});
    let mut v : Vec<Box<dyn IAnyData>> = Vec::new();
    v.push(rect);
    v.push(circle);
    for i in v.iter() {
    if let Some(s) = i.as_any().downcast_ref::<Rectangle>() {
        println!("downcast - Rectangle w={}, h={}", s.width, s.height);
    }else if let Some(s)=i.as_any().downcast_ref::<Circle>() {
            println!("downcast - Circle x={}, y={}, r={}", s.x, s.y, s.radius);
        }else{
            println!("invaild type");
        }
    }
}