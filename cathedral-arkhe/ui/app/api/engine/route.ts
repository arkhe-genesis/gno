import { NextResponse } from 'next/server';

export async function POST(req: Request) {
  const { prompt } = await req.json();
  // Simula chamada ao AGI Core
  return NextResponse.json({
    response: `Processed: ${prompt} (simulated)`,
  });
}
