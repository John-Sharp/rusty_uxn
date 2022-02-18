use std::collections::HashMap;

fn main() {
    let l = vec!("bill".to_owned(), "ben".to_owned(), "bill".to_owned(), "bill".to_owned(), "beth".to_owned());
    let mut m = HashMap::new();

    let l = l.into_iter().map(|e| {
        return e;
    });

    let l = thingy(l.into_iter(), &mut m);

    let _l = l.collect::<Vec<_>>();

    println!("{:?}", m);
}

fn thingy<'a, W:'a>(i: W, m: &'a mut HashMap<String, u32>) -> impl Iterator<Item=String> + 'a
where W: Iterator<Item=String>
{
   i.filter(|e| {
       if e == "beth" {
           return false;
       }
       m.insert(e.clone(), 44);
       return true;
   })
}
