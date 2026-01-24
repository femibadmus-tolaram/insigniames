import sqlite3

db_path = "data/local.db"
tables = ["output_rolls", "input_rolls", "jobs"]

conn = sqlite3.connect(db_path)
cur = conn.cursor()

for table in tables:
    cur.execute(f"DELETE FROM {table}")
    print(f"Emptied table: {table}")

conn.commit()
conn.close()
print("All specified tables have been emptied.")