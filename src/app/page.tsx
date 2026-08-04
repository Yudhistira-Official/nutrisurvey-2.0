'use client';

import { useEffect, useState } from 'react';
import { invokeCommand } from '../lib/commands';

export default function Home() {
  const [status, setStatus] = useState('Checking native bridge…');

  useEffect(() => {
    invokeCommand<string>('ping')
      .then(setStatus)
      .catch(() => setStatus('Native bridge unavailable'));
  }, []);

  return (
    <main>
      <h1>NutriSurvey</h1>
      <p>{status}</p>
    </main>
  );
}
