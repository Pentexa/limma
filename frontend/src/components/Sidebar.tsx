'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  Globe, Server, Search, Layers, Lock, FileCode, BookOpen,
  LayoutDashboard, ChevronRight, Send, Activity, LogOut
} from 'lucide-react';
import { logout } from '@/lib/auth';

const navItems = [
  { label: 'Reconnaissance', isSection: true },
  { href: '/', label: 'Dashboard', icon: LayoutDashboard, description: 'Command center' },
  { href: '/scanner', label: 'Website Scanner', icon: Globe, description: 'Deep web analysis' },
  { href: '/investigator', label: 'Server Investigator', icon: Server, description: 'Fingerprinting' },
  { label: 'Discovery', isSection: true },
  { href: '/api-discovery', label: 'API Discovery', icon: Search, description: 'Endpoint detection' },
  { href: '/services', label: 'Service Collector', icon: Layers, description: 'Port scanning' },
  { href: '/forms', label: 'Form Mapper', icon: FileCode, description: 'Input discovery' },
  { label: 'Security', isSection: true },
  { href: '/audit', label: 'Security Audit', icon: Lock, description: 'Risk assessment' },
  { href: '/rules', label: 'Rule Engine', icon: BookOpen, description: 'Dynamic rules' },
  { label: 'Tools', isSection: true },
  { href: '/proxy', label: 'HTTP Proxy', icon: Send, description: 'Request tester' },
];

/* Limma eye logo as a React component for clean reuse */
function LimmaLogo({ size = 32 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="92 128 839 508"
      fill="none"
      width={size}
      height={size * 0.6}
      style={{ display: 'block', flexShrink: 0 }}
    >
      <g fill="url(#limmaLogoGrad)" fillRule="evenodd">
        <path d="M 102 377 L 112 401 L 162 466 L 221 522 L 273 559 L 326 587 L 384 609 L 486 626 L 552 625 L 613 615 L 681 593 L 744 561 L 800 522 L 840 486 L 873 450 L 921 385 L 902 350 L 857 295 L 817 256 L 759 212 L 714 186 L 638 156 L 571 142 L 526 138 L 416 148 L 363 163 L 293 194 L 258 214 L 198 261 L 157 303 Z M 609 484 L 609 487 L 606 490 L 605 490 L 605 491 L 602 494 L 601 494 L 599 496 L 598 496 L 595 499 L 594 499 L 592 501 L 591 501 L 588 504 L 587 504 L 585 506 L 584 506 L 580 510 L 579 510 L 578 511 L 577 511 L 576 512 L 575 512 L 574 513 L 572 513 L 571 514 L 570 514 L 569 515 L 568 515 L 567 516 L 566 516 L 565 517 L 564 517 L 563 518 L 561 518 L 559 520 L 556 520 L 555 521 L 554 521 L 553 522 L 551 522 L 550 523 L 548 523 L 547 522 L 547 519 L 548 518 L 549 518 L 549 517 L 552 514 L 552 512 L 553 511 L 553 508 L 555 506 L 555 505 L 556 504 L 556 502 L 557 501 L 557 499 L 558 498 L 558 497 L 559 496 L 559 494 L 560 493 L 560 491 L 561 490 L 561 489 L 563 487 L 563 485 L 566 482 L 567 482 L 570 479 L 572 479 L 573 478 L 574 478 L 575 477 L 592 477 L 593 478 L 595 478 L 596 479 L 597 479 L 598 480 L 600 480 L 601 481 L 603 481 L 604 482 L 605 482 L 606 483 L 608 483 Z M 609 443 L 613 443 L 614 442 L 625 442 L 626 443 L 632 443 L 633 444 L 636 444 L 637 445 L 639 445 L 640 446 L 640 449 L 638 451 L 638 452 L 637 453 L 637 454 L 633 458 L 633 459 L 631 461 L 631 462 L 629 464 L 629 465 L 627 467 L 627 468 L 623 472 L 621 472 L 620 471 L 619 471 L 618 470 L 615 470 L 614 469 L 612 469 L 611 468 L 608 468 L 606 466 L 604 466 L 603 465 L 603 461 L 602 460 L 602 455 L 604 453 L 604 452 L 605 451 L 605 448 L 606 447 L 607 447 L 607 445 Z M 589 412 L 592 416 L 592 420 L 594 422 L 597 428 L 597 435 L 591 444 L 591 446 L 588 450 L 586 450 L 586 451 L 581 455 L 581 458 L 579 461 L 578 461 L 576 463 L 573 463 L 572 464 L 567 464 L 562 467 L 560 467 L 558 468 L 552 474 L 552 476 L 549 480 L 548 485 L 544 492 L 544 494 L 541 499 L 541 501 L 539 505 L 535 510 L 535 511 L 533 512 L 532 514 L 530 515 L 521 524 L 518 524 L 517 523 L 517 464 L 518 463 L 522 463 L 523 462 L 526 462 L 527 461 L 532 461 L 535 459 L 539 459 L 540 458 L 542 458 L 544 456 L 546 456 L 554 452 L 556 450 L 559 449 L 559 448 L 561 447 L 567 441 L 568 441 L 569 439 L 575 433 L 575 432 L 577 431 L 577 429 L 579 427 L 580 427 L 581 425 L 581 422 L 584 418 L 586 413 L 587 412 Z M 604 404 L 607 401 L 609 401 L 610 400 L 612 400 L 613 399 L 614 399 L 615 398 L 617 398 L 618 397 L 619 397 L 620 396 L 622 396 L 624 394 L 625 394 L 626 393 L 628 393 L 630 391 L 632 391 L 634 389 L 635 389 L 636 388 L 639 388 L 640 387 L 655 387 L 656 388 L 656 390 L 657 391 L 657 394 L 656 395 L 656 403 L 655 404 L 655 407 L 654 408 L 654 414 L 653 415 L 653 417 L 652 418 L 652 419 L 651 420 L 651 421 L 650 422 L 650 425 L 649 426 L 649 427 L 648 428 L 648 430 L 646 432 L 642 432 L 641 431 L 640 431 L 639 430 L 633 430 L 632 429 L 631 429 L 630 428 L 623 428 L 617 422 L 614 422 L 613 421 L 612 421 L 610 419 L 610 418 L 609 418 L 608 417 L 608 416 L 607 415 L 607 413 L 606 412 L 606 411 L 605 410 L 605 408 L 604 407 Z M 368 387 L 427 387 L 431 389 L 432 401 L 438 419 L 447 432 L 448 435 L 452 438 L 454 442 L 457 443 L 461 448 L 463 448 L 480 459 L 485 459 L 489 461 L 493 461 L 497 463 L 503 463 L 504 464 L 504 527 L 503 528 L 491 528 L 486 526 L 480 526 L 478 524 L 465 521 L 457 518 L 451 514 L 448 514 L 447 512 L 440 509 L 432 504 L 431 502 L 425 499 L 409 485 L 399 474 L 395 468 L 393 467 L 386 456 L 386 454 L 383 451 L 381 445 L 377 441 L 377 438 L 371 425 L 371 422 L 369 418 L 367 406 L 365 401 L 364 390 Z M 511 380 L 511 378 L 512 377 L 513 377 L 518 372 L 521 371 L 521 370 L 522 369 L 527 367 L 529 365 L 531 365 L 534 363 L 537 363 L 538 362 L 540 362 L 541 361 L 547 361 L 548 360 L 551 360 L 552 359 L 565 359 L 566 360 L 568 360 L 569 361 L 577 361 L 578 362 L 580 362 L 581 363 L 585 363 L 588 365 L 591 365 L 592 366 L 594 366 L 595 367 L 599 367 L 604 370 L 606 370 L 607 371 L 609 371 L 611 373 L 613 373 L 615 375 L 618 375 L 619 376 L 619 379 L 618 380 L 616 380 L 614 382 L 612 382 L 610 384 L 608 384 L 606 386 L 604 386 L 603 387 L 601 387 L 600 388 L 597 388 L 595 390 L 594 390 L 593 391 L 590 391 L 589 392 L 586 392 L 585 393 L 583 393 L 582 394 L 580 394 L 579 395 L 577 395 L 576 396 L 572 396 L 569 398 L 548 398 L 547 397 L 544 397 L 543 396 L 541 396 L 540 395 L 538 395 L 537 394 L 534 394 L 531 392 L 529 392 L 527 390 L 525 390 L 522 387 L 517 385 L 515 383 L 515 382 L 513 382 Z M 517 239 L 520 236 L 525 236 L 526 237 L 532 237 L 537 239 L 542 239 L 545 241 L 549 241 L 551 243 L 561 245 L 563 247 L 570 249 L 577 254 L 586 258 L 589 261 L 592 262 L 603 270 L 607 275 L 609 275 L 632 302 L 640 316 L 640 318 L 643 321 L 643 323 L 647 330 L 647 332 L 649 335 L 649 338 L 654 350 L 654 357 L 656 362 L 656 373 L 655 374 L 646 374 L 644 372 L 640 371 L 638 368 L 629 364 L 627 362 L 620 360 L 618 358 L 609 356 L 607 354 L 604 354 L 601 352 L 597 352 L 587 349 L 575 329 L 558 314 L 556 314 L 552 310 L 549 310 L 547 308 L 532 302 L 527 302 L 526 301 L 519 300 L 517 298 Z M 502 237 L 504 239 L 504 297 L 502 300 L 497 300 L 495 302 L 489 302 L 485 304 L 482 304 L 480 306 L 477 306 L 475 308 L 473 308 L 471 310 L 466 312 L 455 320 L 450 325 L 447 330 L 444 332 L 442 338 L 440 339 L 440 341 L 438 343 L 438 346 L 434 352 L 434 356 L 432 358 L 431 371 L 428 374 L 367 374 L 365 372 L 365 364 L 367 359 L 367 353 L 369 350 L 369 345 L 371 342 L 373 333 L 380 319 L 380 317 L 383 314 L 383 311 L 388 306 L 388 304 L 391 301 L 395 294 L 397 293 L 399 289 L 418 270 L 431 260 L 435 259 L 439 255 L 441 255 L 443 253 L 447 252 L 455 247 L 471 241 L 474 241 L 478 239 L 484 239 L 489 237 L 497 237 L 498 236 Z M 388 215 L 391 215 L 392 218 L 375 230 L 350 256 L 332 283 L 316 321 L 310 349 L 308 398 L 310 400 L 310 414 L 315 438 L 332 480 L 361 520 L 373 532 L 394 547 L 394 551 L 350 534 L 288 498 L 244 463 L 211 430 L 174 383 L 178 373 L 208 336 L 241 303 L 265 283 L 305 255 L 336 237 Z M 627 213 L 663 226 L 697 243 L 751 279 L 807 330 L 848 380 L 840 394 L 797 444 L 751 484 L 723 504 L 682 528 L 631 550 L 628 547 L 657 523 L 671 507 L 696 466 L 711 419 L 713 361 L 705 320 L 687 280 L 670 255 L 651 235 L 627 217 Z" />
      </g>
      <defs>
        <linearGradient id="limmaLogoGrad" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#00d4ff" />
          <stop offset="100%" stopColor="#8b5cf6" />
        </linearGradient>
      </defs>
    </svg>
  );
}

export default function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="sidebar">
      {/* Subtle animated scan line */}
      <div style={{
        position: 'absolute', top: 0, left: 0, width: '100%', height: '100%',
        overflow: 'hidden', pointerEvents: 'none', zIndex: 0,
      }}>
        <div style={{
          position: 'absolute', top: '-100%', left: 0, width: '100%', height: '100%',
          background: 'linear-gradient(180deg, transparent 0%, transparent 45%, rgba(0, 212, 255, 0.03) 50%, transparent 55%, transparent 100%)',
          animation: 'scanLine 12s linear infinite',
        }} />
      </div>
      <style>{`
        @keyframes scanLine {
          0% { top: -100%; }
          100% { top: 100%; }
        }
      `}</style>

      {/* Logo area */}
      <div className="sidebar-logo" style={{ position: 'relative', zIndex: 1 }}>
        <LimmaLogo size={36} />
        <div style={{ minWidth: 0 }}>
          <div className="sidebar-logo-text">LIMMA</div>
          <div className="sidebar-logo-version">v0.1.0 • Security Platform</div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="sidebar-nav" style={{ position: 'relative', zIndex: 1 }}>
        {navItems.map((item, i) => {
          if (item.isSection) {
            return (
              <div key={i} className="sidebar-section-label">
                {item.label}
              </div>
            );
          }
          const Icon = item.icon!;
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.href}
              href={item.href!}
              className={`sidebar-link ${isActive ? 'active' : ''}`}
            >
              <Icon size={17} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="sidebar-link-label">{item.label}</div>
                {isActive && (
                  <div className="sidebar-link-desc">
                    {item.description}
                  </div>
                )}
              </div>
              {isActive && (
                <ChevronRight size={14} style={{ opacity: 0.5, flexShrink: 0 }} />
              )}
            </Link>
          );
        })}
        
        {/* Logout Button */}
        <div style={{ marginTop: 'auto', paddingTop: '10px' }}>
          <button 
            onClick={logout}
            className="sidebar-link"
            style={{ width: '100%', background: 'transparent', border: 'none', textAlign: 'left', cursor: 'pointer' }}
          >
            <LogOut size={17} style={{ color: 'var(--color-danger)' }} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="sidebar-link-label" style={{ color: 'var(--color-danger)' }}>Çıkış Yap</div>
            </div>
          </button>
        </div>
      </nav>

      {/* Footer */}
      <div className="sidebar-footer" style={{ position: 'relative', zIndex: 1 }}>
        <div style={{
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          gap: 6, marginBottom: 4,
        }}>
          <Activity size={10} style={{ color: 'var(--color-success)', opacity: 0.8 }} />
          <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', letterSpacing: '0.04em' }}>System Online</span>
        </div>
        <span style={{ opacity: 0.35, fontSize: '0.6rem', letterSpacing: '0.03em' }}>© 2026 Limma Security</span>
      </div>
    </aside>
  );
}
