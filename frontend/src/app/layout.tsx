import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'
import { Sidebar } from '@/components/Sidebar'
import { LanguageProvider } from '@/context/LanguageContext'

const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'Limma | Web Intelligence OS',
  description: 'Premium cybersecurity and web intelligence tool',
  icons: {
    icon: '/logo.png',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={`${inter.className} bg-background text-foreground antialiased min-h-screen relative overflow-x-hidden selection:bg-accent-cyan/30 selection:text-white`}>
        <LanguageProvider>
          <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-accent-blue/5 rounded-full blur-3xl pointer-events-none -z-10 translate-x-1/2 -translate-y-1/2"></div>
          <Sidebar />
          <div className="pl-64 flex flex-col min-h-screen">
            <main className="flex-1 max-w-7xl mx-auto w-full p-8 pt-10 relative">
              {children}
            </main>
          </div>
        </LanguageProvider>
      </body>
    </html>
  )
}
