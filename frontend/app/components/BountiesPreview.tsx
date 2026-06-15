import { ExternalLink } from "lucide-react";

export default function BountiesPreview() {
  const bounties = [
    { title: "Implement Freighter Wallet SDK", amount: "500 USDC", tags: ["Frontend", "React"] },
    { title: "Write Soroban Tests for Escrow", amount: "1,200 XLM", tags: ["Rust", "Smart Contract"] },
    { title: "Design Landing Page Assets", amount: "300 USDC", tags: ["Design", "Figma"] }
  ];

  return (
    <section className="w-full py-20 bg-neutral-950/50 border-y border-white/5 z-10">
      <div className="max-w-6xl mx-auto px-6">
        <div className="flex justify-between items-end mb-12">
          <div>
            <h2 className="text-3xl font-bold text-white mb-2">Live Bounties</h2>
            <p className="text-neutral-400">Earn crypto directly to your Stellar wallet.</p>
          </div>
          <button className="text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-2">
            View All <ExternalLink className="w-4 h-4" />
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {bounties.map((b, i) => (
            <div key={i} className="p-6 rounded-2xl bg-neutral-900 border border-neutral-800 hover:border-indigo-500/50 transition-all cursor-pointer group">
              <div className="flex justify-between items-start mb-4">
                <span className="px-3 py-1 bg-indigo-500/10 text-indigo-400 text-xs font-bold rounded-full">
                  {b.amount}
                </span>
              </div>
              <h3 className="text-lg font-bold text-white mb-4 group-hover:text-indigo-300 transition-colors">{b.title}</h3>
              <div className="flex gap-2">
                {b.tags.map(t => (
                  <span key={t} className="text-xs text-neutral-500 bg-neutral-800 px-2 py-1 rounded-md">{t}</span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
