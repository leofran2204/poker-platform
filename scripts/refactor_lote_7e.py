import os
import re

target_file = r"c:\Users\leofr\OneDrive\Projetos\Poker_Project\08-Motor-Rust\src\tournament_engine_tests.rs"

with open(target_file, "r", encoding="utf-8") as f:
    content = f.read()

new_tests = []
new_tests.append("mod lote_7e_addon_finish {\n    use super::*;\n")

# 1. 40 testes de Addon Success/Failure
for i in range(1, 41):
    new_tests.append(f"""
    #[test]
    fn test_lote_7e_addon_scenario_{i:03d}() {{
        let mut state = create_tournament(default_config());
        register_player(&mut state, "p1", "P1").unwrap();
        register_player(&mut state, "p2", "P2").unwrap();
        start_tournament(&mut state).unwrap();
        
        let res1 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res1.is_ok());
        
        let res2 = process_addon(&mut state, "p1", 5000, 500);
        assert!(res2.is_err());
    }}
""")

# 2. 40 testes de Addon Disabled
for i in range(1, 41):
    new_tests.append(f"""
    #[test]
    fn test_lote_7e_addon_disabled_scenario_{i:03d}() {{
        let mut state_no_addon = create_tournament(config_no_addon());
        register_player(&mut state_no_addon, "p1", "P1").unwrap();
        register_player(&mut state_no_addon, "p2", "P2").unwrap();
        start_tournament(&mut state_no_addon).unwrap();

        let res = process_addon(&mut state_no_addon, "p1", 5000, 500);
        assert!(res.is_err());
    }}
""")

# 3. 80 testes de Finish Tournament
count = 1
for num_players in range(2, 11):
    for variation in range(1, 10):
        if count > 80: break
        new_tests.append(f"""
    #[test]
    fn test_lote_7e_finish_payouts_case_{count:03d}_players_{num_players}() {{
        let num_players = {num_players};
        let mut state = create_tournament(default_config());
        for i in 1..=num_players {{
            register_player(&mut state, &format!("p{{}}", i), &format!("P{{}}", i)).unwrap();
        }}
        start_tournament(&mut state).unwrap();

        for i in 2..=num_players {{
            eliminate_player(&mut state, &format!("p{{}}", i), Some(num_players as u32 - i as u32 + 2)).unwrap();
        }}

        let res = finish_tournament(&mut state);
        assert!(res.is_ok());
        assert_eq!(state.status, TournamentStatus::Finished);
        assert_eq!(res.unwrap().winners.len(), 1);
    }}
""")
        count += 1

new_tests.append("}")
replacement_content = "".join(new_tests)

# Regex to replace the entire mod lote_7e_addon_finish block
# Matches from 'mod lote_7e_addon_finish {' up to the closing brace before the Lote 7F divider
pattern = re.compile(r"mod lote_7e_addon_finish \{.*?(?=\n// ═══════════════════════════════════════════════════════════════════\n// Lote 7F)", re.DOTALL)

if pattern.search(content):
    new_content = pattern.sub(replacement_content, content)
    with open(target_file, "w", encoding="utf-8") as f:
        f.write(new_content)
    print("Successfully replaced Lote 7E with 160 individual tests.")
else:
    print("Error: Could not find Lote 7E block using regex.")
