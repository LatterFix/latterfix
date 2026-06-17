"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import { Search, Filter, MapPin, Clock, Star, ArrowRight, Grid, List } from "lucide-react";

// ---------------------------------------------------------------------------
// Mock bounty data — replace with real contract fetch when integrated
// ---------------------------------------------------------------------------
interface Bounty {
  id: number;
  title: string;
  description: string;
  reward: number;
  token: string;
  difficulty: "Easy" | "Medium" | "Hard";
  tags: string[];
  postedAt: string;
  applicantCount: number;
  location: string;
}

const MOCK_BOUNTIES: Bounty[] = [
  {
    id: 1,
    title: "Integrate Stripe Checkout for Pro Plans",
    description: "Add Stripe checkout flow to the subscription page. Handle webhook events for successful payments and store subscription state in our database.",
    reward: 850,
    token: "USDC",
    difficulty: "Medium",
    tags: ["TypeScript", "React", "Stripe"],
    postedAt: "2h ago",
    applicantCount: 3,
    location: "Remote",
  },
  {
    id: 2,
    title: "Fix memory leak in WebSocket handler",
    description: "Our production server experiences a gradual memory increase tied to WebSocket connections. Identify and patch the leak in the connection handler.",
    reward: 1200,
    token: "USDC",
    difficulty: "Hard",
    tags: ["Node.js", "WebSocket", "Debugging"],
    postedAt: "5h ago",
    applicantCount: 1,
    location: "Remote",
  },
  {
    id: 3,
    title: "Design onboarding flow wireframes",
    description: "Create wireframes for the new 4-step user onboarding experience. Deliver in Figma with interactive prototypes.",
    reward: 400,
    token: "USDC",
    difficulty: "Easy",
    tags: ["Design", "Figma", "UX"],
    postedAt: "1d ago",
    applicantCount: 7,
    location: "Remote",
  },
  {
    id: 4,
    title: "Write API documentation for v2 endpoints",
    description: "Document all new v2 REST endpoints using OpenAPI 3.1 spec. Include request/response examples and error codes.",
    reward: 600,
    token: "USDC",
    difficulty: "Medium",
    tags: ["Docs", "OpenAPI", "REST"],
    postedAt: "1d ago",
    applicantCount: 2,
    location: "Remote",
  },
  {
    id: 5,
    title: "Optimize Postgres query for activity feed",
    description: "The /feed endpoint takes 4s to load for users with >10k followers. Rewrite the query using proper indexes and pagination.",
    reward: 950,
    token: "USDC",
    difficulty: "Hard",
    tags: ["PostgreSQL", "Performance", "SQL"],
    postedAt: "3d ago",
    applicantCount: 0,
    location: "Remote",
  },
  {
    id: 6,
    title: "Add dark mode toggle to settings page",
    description: "Implement a dark/light mode toggle in the user settings. Persist preference in localStorage and respect system preference as default.",
    reward: 200,
    token: "USDC",
    difficulty: "Easy",
    tags: ["CSS", "React", "Tailwind"],
    postedAt: "6h ago",
    applicantCount: 5,
    location: "Remote",
  },
];

const DIFFICULTY_COLORS = {
  Easy: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
  Medium: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
  Hard: "bg-red-500/10 text-red-400 border-red-500/20",
};

export default function BountiesPage() {
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [search, setSearch] = useState("");
  const [selectedDifficulty, setSelectedDifficulty] = useState<string>("All");

  const filtered = MOCK_BOUNTIES.filter((b) => {
    const matchesSearch =
      b.title.toLowerCase().includes(search.toLowerCase()) ||
      b.description.toLowerCase().includes(search.toLowerCase()) ||
      b.tags.some((t) => t.toLowerCase().includes(search.toLowerCase()));
    const matchesDiff =
      selectedDifficulty === "All" || b.difficulty === selectedDifficulty;
    return matchesSearch && matchesDiff;
  });

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-50">
      {/* Header */}
      <div className="border-b border-indigo-900/30">
        <div className="max-w-7xl mx-auto px-6 py-8">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div>
              <h1 className="text-3xl font-extrabold tracking-tight">
                Explore Bounties
              </h1>
              <p className="text-neutral-400 mt-1">
                {filtered.length} task{filtered.length !== 1 ? "s" : ""} available
              </p>
            </div>
            <button className="px-6 py-3 rounded-full bg-indigo-600 text-white font-semibold hover:bg-indigo-500 transition-all flex items-center gap-2 shadow-[0_0_20px_rgba(79,70,229,0.4)]">
              Post a Bounty <ArrowRight className="w-4 h-4" />
            </button>
          </div>

          {/* Search + filters */}
          <div className="flex flex-col sm:flex-row gap-3 mt-6">
            <div className="relative flex-1">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-neutral-500" />
              <input
                type="text"
                placeholder="Search by title, description, or skill..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="w-full pl-11 pr-4 py-2.5 rounded-xl bg-neutral-900 border border-neutral-800 text-neutral-100 placeholder-neutral-500 text-sm focus:outline-none focus:border-indigo-500 transition-colors"
              />
            </div>
            <div className="flex items-center gap-2">
              {["All", "Easy", "Medium", "Hard"].map((d) => (
                <button
                  key={d}
                  onClick={() => setSelectedDifficulty(d)}
                  className={`px-4 py-2 rounded-xl text-sm font-medium border transition-all ${
                    selectedDifficulty === d
                      ? "bg-indigo-600 border-indigo-600 text-white"
                      : "bg-neutral-900 border-neutral-800 text-neutral-400 hover:border-neutral-700"
                  }`}
                >
                  {d}
                </button>
              ))}
            </div>
            <div className="flex items-center border border-neutral-800 rounded-xl overflow-hidden">
              <button
                onClick={() => setViewMode("grid")}
                className={`p-2.5 transition-colors ${
                  viewMode === "grid" ? "bg-indigo-600 text-white" : "bg-neutral-900 text-neutral-500 hover:text-white"
                }`}
              >
                <Grid className="w-4 h-4" />
              </button>
              <button
                onClick={() => setViewMode("list")}
                className={`p-2.5 transition-colors ${
                  viewMode === "list" ? "bg-indigo-600 text-white" : "bg-neutral-900 text-neutral-500 hover:text-white"
                }`}
              >
                <List className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Bounties grid / list */}
      <div className="max-w-7xl mx-auto px-6 py-10">
        {filtered.length === 0 ? (
          <div className="text-center py-24">
            <p className="text-neutral-500 text-lg">No bounties match your search.</p>
            <button
              onClick={() => { setSearch(""); setSelectedDifficulty("All"); }}
              className="mt-4 text-indigo-400 hover:text-indigo-300 text-sm font-medium"
            >
              Clear filters
            </button>
          </div>
        ) : viewMode === "grid" ? (
          <motion.div
            initial="hidden"
            animate="visible"
            variants={{ visible: { transition: { staggerChildren: 0.06 } } }}
            className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6"
          >
            {filtered.map((bounty) => (
              <BountyCard key={bounty.id} bounty={bounty} />
            ))}
          </motion.div>
        ) : (
          <motion.div
            initial="hidden"
            animate="visible"
            variants={{ visible: { transition: { staggerChildren: 0.04 } } }}
            className="flex flex-col gap-4"
          >
            {filtered.map((bounty) => (
              <BountyListItem key={bounty.id} bounty={bounty} />
            ))}
          </motion.div>
        )}
      </div>
    </div>
  );
}

function BountyCard({ bounty }: { bounty: Bounty }) {
  return (
    <motion.div
      variants={{
        hidden: { opacity: 0, y: 16 },
        visible: { opacity: 1, y: 0, transition: { duration: 0.35 } },
      }}
      whileHover={{ y: -4, transition: { duration: 0.2 } }}
      className="group relative flex flex-col rounded-2xl bg-neutral-900/70 border border-neutral-800 p-6 hover:border-indigo-500/50 transition-all cursor-pointer shadow-[0_0_0_1px_rgba(0,0,0,0)] hover:shadow-[0_0_25px_rgba(79,70,229,0.15)]"
    >
      {/* Top row: difficulty badge + posted time */}
      <div className="flex items-center justify-between mb-4">
        <span className={`text-xs font-semibold px-2.5 py-1 rounded-full border ${DIFFICULTY_COLORS[bounty.difficulty]}`}>
          {bounty.difficulty}
        </span>
        <span className="text-xs text-neutral-500 flex items-center gap-1">
          <Clock className="w-3 h-3" /> {bounty.postedAt}
        </span>
      </div>

      {/* Title */}
      <h3 className="font-semibold text-neutral-100 text-base leading-snug mb-2 line-clamp-2 group-hover:text-white transition-colors">
        {bounty.title}
      </h3>

      {/* Description */}
      <p className="text-sm text-neutral-400 leading-relaxed mb-4 line-clamp-3 flex-1">
        {bounty.description}
      </p>

      {/* Tags */}
      <div className="flex flex-wrap gap-2 mb-5">
        {bounty.tags.map((tag) => (
          <span
            key={tag}
            className="text-xs px-2.5 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20"
          >
            {tag}
          </span>
        ))}
      </div>

      {/* Footer: reward + applicants */}
      <div className="flex items-center justify-between pt-4 border-t border-neutral-800">
        <div>
          <p className="text-xs text-neutral-500">Reward</p>
          <p className="font-mono text-lg font-bold text-emerald-400">
            {bounty.reward.toLocaleString()} {bounty.token}
          </p>
        </div>
        <div className="text-right">
          <p className="text-xs text-neutral-500">Applicants</p>
          <p className="font-mono text-sm font-medium text-neutral-300">
            {bounty.applicantCount}
          </p>
        </div>
        <button className="px-4 py-2 rounded-xl bg-indigo-600 text-white text-sm font-semibold hover:bg-indigo-500 transition-all flex items-center gap-1.5 shadow-[0_0_15px_rgba(79,70,229,0.3)]">
          View <ArrowRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </motion.div>
  );
}

function BountyListItem({ bounty }: { bounty: Bounty }) {
  return (
    <motion.div
      variants={{
        hidden: { opacity: 0, x: -8 },
        visible: { opacity: 1, x: 0, transition: { duration: 0.3 } },
      }}
      whileHover={{ borderColor: "rgba(79,70,229,0.5)" }}
      className="flex items-center gap-5 rounded-2xl bg-neutral-900/70 border border-neutral-800 p-5 hover:bg-neutral-900 transition-all cursor-pointer"
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-3 mb-1.5">
          <span className={`text-xs font-semibold px-2.5 py-0.5 rounded-full border ${DIFFICULTY_COLORS[bounty.difficulty]}`}>
            {bounty.difficulty}
          </span>
          <span className="text-xs text-neutral-500 flex items-center gap-1">
            <Clock className="w-3 h-3" /> {bounty.postedAt}
          </span>
          <span className="text-xs text-neutral-500 flex items-center gap-1">
            <MapPin className="w-3 h-3" /> {bounty.location}
          </span>
        </div>
        <h3 className="font-semibold text-neutral-100 text-sm leading-snug mb-1">
          {bounty.title}
        </h3>
        <div className="flex flex-wrap gap-1.5">
          {bounty.tags.map((tag) => (
            <span key={tag} className="text-xs px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
              {tag}
            </span>
          ))}
        </div>
      </div>
      <div className="text-right flex-shrink-0">
        <p className="font-mono text-sm font-bold text-emerald-400">
          {bounty.reward.toLocaleString()} {bounty.token}
        </p>
        <p className="text-xs text-neutral-500 mt-0.5">{bounty.applicantCount} applicant{bounty.applicantCount !== 1 ? "s" : ""}</p>
      </div>
    </motion.div>
  );
}
