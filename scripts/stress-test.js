// scripts/stress-test.js — Production k6 Stress & Load Testing Script
//
// Bu betik, canlı sunucumuzun yüksek trafik altındaki davranışını ve dayanıklılığını
// ölçmek üzere tasarlanmış bir k6 stress testidir.
//
// Çalıştırmak için:
//   k6 run --env TARGET_URL=https://your-load-balancer-dns.com scripts/stress-test.js
//

import http from 'k6/http';
import { check, sleep } from 'k6';

// 1. Stress Test Senaryosu ve Yük Aşamaları (Ramp-up / Peak / Cool-down)
export const options = {
  stages: [
    { duration: '30s', target: 50 },  // 30 saniyede 50 sanal kullanıcıya yüksel (Ramp-up)
    { duration: '1m', target: 50 },   // 1 dakika boyunca 50 kullanıcıyla kararlı yük testi
    { duration: '30s', target: 200 }, // 30 saniyede 200 sanal kullanıcıya sıçra (Stress Spike)
    { duration: '1m', target: 200 },  // 1 dakika boyunca 200 kullanıcıyla yükü sürdür (Peak Load)
    { duration: '30s', target: 0 },   // 30 saniyede yükü sıfıra indir (Cool-down / Recovery)
  ],
  thresholds: {
    // Güvenlik ve SLA Performans Eşikleri
    http_req_failed: ['rate<0.01'],    // Hata oranı %1'in altında olmalıdır
    http_req_duration: ['p(95)<300'], // İsteklerin %95'i 300ms'den kısa sürmelidir
  },
};

// Target URL default fallback
const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

// 2. Sanal Kullanıcı (VU) İş Akışı
export default function () {
  const params = {
    headers: {
      'User-Agent': 'k6-load-testing-agent',
      'Content-Type': 'application/json',
    },
  };

  // Senaryo A: Ana Sayfa Yükleme
  const homeRes = http.get(`${BASE_URL}/`, params);
  check(homeRes, {
    'Home page status is 200': (r) => r.status === 200,
    'Home page response time < 200ms': (r) => r.timings.duration < 200,
  });
  sleep(1);

  // Senaryo B: Login Sayfası Yükleme
  const loginRes = http.get(`${BASE_URL}/login`, params);
  check(loginRes, {
    'Login page status is 200': (r) => r.status === 200,
  });
  sleep(1.5);

  // Senaryo C: Sağlık Kontrolü (Health Check)
  const healthRes = http.get(`${BASE_URL}/health`, params);
  check(healthRes, {
    'Health check status is 200': (r) => r.status === 200,
  });
  sleep(2);
}
