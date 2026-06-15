const express = require("express");
const cors = require("cors");
const bodyParser = require("body-parser");
const sqlite3 = require("sqlite3").verbose();
const { Horizon } = require("@stellar/stellar-sdk");

const app = express();
const PORT = process.env.PORT || 3001;

app.use(cors());
app.use(bodyParser.json());

// Initialize SQLite database
const db = new sqlite3.Database("./tasks.db", (err) => {
  if (err) {
    console.error("Failed to connect to database:", err.message);
  } else {
    console.log("Connected to SQLite database.");
    db.run(`
      CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        reward TEXT NOT NULL,
        token TEXT NOT NULL,
        status TEXT NOT NULL,
        assignee TEXT,
        creator TEXT NOT NULL,
        submission TEXT,
        tags TEXT
      )
    `, () => {
      // Seed data if database is empty
      db.get("SELECT COUNT(*) as count FROM tasks", (err, row) => {
        if (row && row.count === 0) {
          const stmt = db.prepare(`
            INSERT INTO tasks (title, description, reward, token, status, creator, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?)
          `);
          stmt.run("Implement Freighter Wallet SDK", "Integrate freighter-api library to support web-based signing.", "500", "USDC", "Open", "GDAV...CREATOR", "Frontend,React");
          stmt.run("Write Soroban Tests for Escrow", "Implement robust integration test scenarios for the payout escrow.", "1200", "XLM", "Open", "GDAV...CREATOR", "Rust,Smart Contract");
          stmt.run("Design Landing Page Assets", "Create Figma assets and mockups for the dashboard layout.", "300", "USDC", "Open", "GDAV...CREATOR", "Design,Figma");
          stmt.finalize();
          console.log("Database seeded with default tasks.");
        }
      });
    });
  }
});

// Stellar Integration: Setup Horizon Server Listener
const server = new Horizon.Server("https://horizon-testnet.stellar.org");

// A utility to verify a transaction status on Stellar Horizon
async function checkStellarTx(txHash) {
  try {
    const tx = await server.transactionDetail(txHash);
    return tx.successful;
  } catch (error) {
    console.error("Stellar verification failed:", error);
    return false;
  }
}

// REST Endpoints
app.get("/api/tasks", (req, res) => {
  db.all("SELECT * FROM tasks", [], (err, rows) => {
    if (err) {
      res.status(500).json({ error: err.message });
      return;
    }
    res.json(rows.map(row => ({
      ...row,
      tags: row.tags ? row.tags.split(",") : []
    })));
  });
});

app.post("/api/tasks", (req, res) => {
  const { title, description, reward, token, creator, tags } = req.body;
  if (!title || !description || !reward || !creator) {
    return res.status(400).json({ error: "Missing required fields" });
  }

  const tagsStr = Array.isArray(tags) ? tags.join(",") : "";
  db.run(
    `INSERT INTO tasks (title, description, reward, token, status, creator, tags) VALUES (?, ?, ?, ?, 'Open', ?, ?)`,
    [title, description, reward, token || "USDC", creator, tagsStr],
    function (err) {
      if (err) {
        return res.status(500).json({ error: err.message });
      }
      res.status(201).json({ id: this.lastID, title, description, reward, token, status: "Open", creator, tags });
    }
  );
});

app.post("/api/tasks/:id/assign", (req, res) => {
  const { id } = req.params;
  const { assignee } = req.body;
  if (!assignee) return res.status(400).json({ error: "Assignee required" });

  db.run(
    `UPDATE tasks SET assignee = ?, status = 'InProgress' WHERE id = ? AND status = 'Open'`,
    [assignee, id],
    function (err) {
      if (err) return res.status(500).json({ error: err.message });
      if (this.changes === 0) return res.status(400).json({ error: "Task not available or already assigned" });
      res.json({ message: "Task assigned successfully", id, assignee, status: "InProgress" });
    }
  );
});

app.post("/api/tasks/:id/submit", (req, res) => {
  const { id } = req.params;
  const { submission } = req.body;
  if (!submission) return res.status(400).json({ error: "Submission details required" });

  db.run(
    `UPDATE tasks SET submission = ?, status = 'Completed' WHERE id = ? AND status = 'InProgress'`,
    [submission, id],
    function (err) {
      if (err) return res.status(500).json({ error: err.message });
      if (this.changes === 0) return res.status(400).json({ error: "Task not in-progress" });
      res.json({ message: "Task work submitted", id, status: "Completed" });
    }
  );
});

app.post("/api/tasks/:id/complete", async (req, res) => {
  const { id } = req.params;
  const { txHash } = req.body; // Stellar transaction hash demonstrating payment release on-chain

  if (txHash) {
    const isTxValid = await checkStellarTx(txHash);
    if (!isTxValid) {
      return res.status(400).json({ error: "Stellar payment transaction not verified on Horizon Testnet" });
    }
  }

  db.run(
    `UPDATE tasks SET status = 'Verified' WHERE id = ? AND status = 'Completed'`,
    [id],
    function (err) {
      if (err) return res.status(500).json({ error: err.message });
      if (this.changes === 0) return res.status(400).json({ error: "Task cannot be verified" });
      res.json({ message: "Task verified and closed.", id, status: "Verified" });
    }
  );
});

app.listen(PORT, () => {
  console.log(`Backend server running on port ${PORT}`);
});
