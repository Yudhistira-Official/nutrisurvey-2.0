'use client';

import { useEffect, useState } from 'react';

const COUNTDOWN = 10;

export default function DonationPopup({ onClose }: { onClose: () => void }) {
  const [seconds, setSeconds] = useState(COUNTDOWN);
  const [closing, setClosing] = useState(false);

  const handleClose = () => {
    setClosing(true);
    setTimeout(onClose, 350);
  };

  useEffect(() => {
    if (seconds <= 0) { handleClose(); return; }
    const t = setTimeout(() => setSeconds(s => s - 1), 1000);
    return () => clearTimeout(t);
  }, [seconds]);

  return (
    <div className={`donation-overlay${closing ? ' closing' : ''}`} aria-modal="true" role="dialog" aria-label="Donasi">
      <div className={`donation-popup${closing ? ' closing' : ''}`}>
        <div className="donation-header">
          <h2>Dukung Project Ini</h2>
          <p>NutriSurvey Pro dibuat dengan sepenuh hati. Jika aplikasi ini bermanfaat, pertimbangkan untuk berdonasi.</p>
        </div>
        <img
          src="/QRIS-Donation.png"
          alt="QRIS Donasi"
          className="donation-qris"
        />
        <div className="donation-footer">
          <p className="donation-note">Scan QRIS di atas untuk berdonasi. Terima kasih atas dukunganmu!</p>
          <button className="donation-close" onClick={handleClose}>
            Tutup ({seconds})
          </button>
        </div>
      </div>
    </div>
  );
}
