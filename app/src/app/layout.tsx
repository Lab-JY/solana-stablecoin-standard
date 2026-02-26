import type { Metadata } from "next";
import { WalletProvider } from "@/components/WalletProvider";
import "./globals.css";

export const metadata: Metadata = {
  title: "SSS - Solana Stablecoin Standard",
  description: "Create and manage Token-2022 stablecoins on Solana",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="bg-solana-dark text-white min-h-screen">
        <WalletProvider>
          <nav className="border-b border-solana-border bg-solana-card/50 backdrop-blur-sm sticky top-0 z-50">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="flex items-center justify-between h-16">
                <div className="flex items-center space-x-8">
                  <a href="/" className="text-xl font-bold">
                    <span className="text-solana-purple">SSS</span>
                    <span className="text-gray-400 text-sm ml-2">
                      Stablecoin Standard
                    </span>
                  </a>
                  <div className="hidden md:flex space-x-4">
                    <a
                      href="/create"
                      className="text-gray-300 hover:text-solana-green transition-colors px-3 py-2 rounded-md text-sm"
                    >
                      Create
                    </a>
                    <a
                      href="/manage"
                      className="text-gray-300 hover:text-solana-green transition-colors px-3 py-2 rounded-md text-sm"
                    >
                      Manage
                    </a>
                    <a
                      href="/compliance"
                      className="text-gray-300 hover:text-solana-green transition-colors px-3 py-2 rounded-md text-sm"
                    >
                      Compliance
                    </a>
                  </div>
                </div>
                <div id="wallet-button" />
              </div>
            </div>
          </nav>
          <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            {children}
          </main>
        </WalletProvider>
      </body>
    </html>
  );
}
