export interface SsqRecord {
  issue: string;
  date: string;
  red_balls: number[];
  blue_ball: number;
}

export interface BallFrequency {
  number: number;
  frequency: number;
  weight: number;
}

export interface PredictionResult {
  red_balls: number[];
  blue_ball: number;
  score: number;
}

export type AlgorithmType = 'hot' | 'cold';
