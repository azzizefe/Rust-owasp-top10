// rate_limit_test.js — k6 Performance & Rate Limit Stress Test Script
//
// Bu betik, Axum uygulamamızın "Tower-Governor" rate limit (Hız Sınırı) mekanizmasını
// ve Nginx çeper yük dengesini stress testine tabi tutar.
//
// Çalıştırma: k6 run rate_limit_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    // 1. Normal Trafik Senaryosu (Hız sınırına takılmamalıdır)
    normal_traffic: {
      executor: 'constant-vus',
      vus: 2,
      duration: '10s',
      exec: 'normalUser',
    },
    // 2. Saldırı/Brute-Force Senaryosu (429 Too Many Requests dönmelidir)
    brute_force_attack: {
      executor: 'constant-arrival-rate',
      rate: 20, // Saniyede 20 istek (Limitimiz 2 req/sec, burst 5)
      timeUnit: '1s',
      duration: '10s',
      preAllocatedVUs: 10,
      maxVUs: 50,
      exec: 'attackerUser',
    },
  },
  thresholds: {
    // Brute force saldırısında HTTP 429 başarı oranı yüksek olmalıdır
    'http_req_failed{scenario:brute_force_attack}': ['rate > 0.50'], 
    // Normal trafikte hiç hata (HTTP 429/500 vb.) olmamalıdır
    'http_req_failed{scenario:normal_traffic}': ['rate == 0.00'], 
  },
};

// TLS sertifika doğrulamalarını yerelde yok say (Self-signed sertifika uyumu)
http.setResponseCallback(null);

export function normalUser() {
  const params = {
    headers: { 'Content-Type': 'application/json' },
    redirects: 0,
    tags: { scenario: 'normal_traffic' },
  };
  
  // Normal kullanıcı: Her 2 saniyede 1 istek atar (Limit içi)
  const res = http.get('https://localhost:443/login', params);
  
  check(res, {
    'normal: status is 200': (r) => r.status === 200,
  });
  
  sleep(2); 
}

export function attackerUser() {
  const params = {
    headers: { 'Content-Type': 'application/json' },
    redirects: 0,
    tags: { scenario: 'brute_force_attack' },
  };

  // Saldırgan: Sınır tanımadan peş peşe istek fırlatır (Limit dışı)
  const res = http.get('https://localhost:443/login', params);

  check(res, {
    'attacker: blocked with 429': (r) => r.status === 429,
  });
}
