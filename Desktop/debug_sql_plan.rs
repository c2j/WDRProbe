use wdrprobe_desktop_lib::parsers::sql_parser::*;

fn main() {
    let sql_plan_text = r#"select * from t1,t2 where t1.c1=t2.c2;
QUERY PLAN
---------------------------------------------------------------------------------
 Streaming (type: GATHER)  (cost=14.17..29.07 rows=20 width=180)
   ->  Hash Join  (cost=14.17..29.07 rows=20 width=180)
         Hash Cond: (t1.c1 = t2.c2)
         ->  Seq Scan on t1  (cost=0.00..12.87 rows=387 width=52)
         ->  Hash  (cost=12.25..12.25 rows=387 width=128)
               ->  Seq Scan on t2  (cost=0.00..12.25 rows=387 width=128)"#;

    println!("Testing SQL+PLAN format parsing...");
    
    let result = parse_sql_plan_format(sql_plan_text);
    match result {
        Ok(plan) => {
            println!("✓ Parse successful");
            println!("Root operation: {}", plan.operation);
            println!("Root children count: {}", plan.children.len());
            
            if !plan.children.is_empty() {
                let child = &plan.children[0];
                println!("Child operation: {}", child.operation);
                println!("Child children count: {}", child.children.len());
                
                if !child.children.is_empty() {
                    let grandchild = &child.children[0];
                    println!("Grandchild operation: {}", grandchild.operation);
                }
            }
            
            if let Some(output) = &plan.node_details.output {
                println!("SQL output: {}", output[0]);
            }
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
        }
    }
}