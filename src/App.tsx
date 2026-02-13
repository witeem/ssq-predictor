import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as echarts from 'echarts';
import type { SsqRecord, BallFrequency, PredictionResult, AlgorithmType } from "./types";

function App() {
  const [loading, setLoading] = useState(false);
  const [records, setRecords] = useState<SsqRecord[]>([]);
  const [algorithm, setAlgorithm] = useState<AlgorithmType>("hot");
  const [redFrequencies, setRedFrequencies] = useState<BallFrequency[]>([]);
  const [blueFrequencies, setBlueFrequencies] = useState<BallFrequency[]>([]);
  const [predictions, setPredictions] = useState<PredictionResult[]>([]);
  const [error, setError] = useState<string>("");
  
  const redBarChartRef = useRef<HTMLDivElement>(null);
  const blueBarChartRef = useRef<HTMLDivElement>(null);
  
  // 防止 StrictMode 重复调用
  const hasLoadedData = useRef(false);

  const loadData = async () => {
    setLoading(true);
    setError("");
    try {
      const data = await invoke<SsqRecord[]>("load_and_update_data");
      setRecords(data);
      await analyzeFrequency(data, algorithm);
    } catch (err) {
      setError(`加载失败: ${err}`);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const analyzeFrequency = async (data: SsqRecord[], algo: AlgorithmType) => {
    try {
      const [redFreq, blueFreq] = await invoke<[BallFrequency[], BallFrequency[]]>(
        "analyze_frequency",
        { records: data, algorithm: algo }
      );
      setRedFrequencies(redFreq);
      setBlueFrequencies(blueFreq);
    } catch (err) {
      setError(`分析失败: ${err}`);
      console.error(err);
    }
  };

  const generatePredictions = async () => {
    if (records.length === 0) {
      setError("请先加载数据");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const result = await invoke<PredictionResult[]>("generate_predictions", {
        records,
        algorithm,
      });
      setPredictions(result);
    } catch (err) {
      setError(`预测失败: ${err}`);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleAlgorithmChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const algo = e.target.value as AlgorithmType;
    setAlgorithm(algo);
    setPredictions([]);
    if (records.length > 0) {
      await analyzeFrequency(records, algo);
    }
  };

  useEffect(() => {
    // 防止 StrictMode 或其他原因导致的重复调用
    if (hasLoadedData.current) {
      return;
    }
    hasLoadedData.current = true;
    loadData();
  }, []);

  // 渲染红球柱形图
  useEffect(() => {
    if (redFrequencies.length > 0 && redBarChartRef.current) {
      // 获取或创建图表实例
      let chart = echarts.getInstanceByDom(redBarChartRef.current);
      if (!chart) {
        chart = echarts.init(redBarChartRef.current, null, {
          renderer: 'canvas', // 使用 canvas 渲染器提升性能
          useDirtyRect: true // 启用脏矩形优化
        });
      }
      
      // 计算总权重用于百分比显示
      const totalWeight = redFrequencies.reduce((sum, f) => sum + f.weight, 0);
      
      const option = {
        animation: false, // 滚动时禁用动画
        title: {
          text: '红球频率柱形图',
          left: 'center',
          textStyle: { fontSize: 16, fontWeight: 'bold' }
        },
        tooltip: {
          trigger: 'axis',
          axisPointer: { type: 'shadow' },
          formatter: (params: any) => {
            const dataIndex = params[0].dataIndex;
            const ball = redFrequencies[dataIndex];
            const weightPercent = totalWeight > 0 ? (ball.weight / totalWeight * 100).toFixed(2) : '0.00';
            return `球号: ${ball.number.toString().padStart(2, '0')}<br/>` +
                   `出现次数: ${ball.frequency}<br/>` +
                   `权重: ${ball.weight.toFixed(2)}<br/>` +
                   `权重占比: ${weightPercent}%`;
          }
        },
        xAxis: {
          type: 'category',
          data: redFrequencies.map(f => f.number.toString().padStart(2, '0')),
          axisLabel: { interval: 0, rotate: 0, fontSize: 12 }
        },
        yAxis: {
          type: 'value',
          name: '频次'
        },
        series: [{
          name: '出现次数',
          type: 'bar',
          data: redFrequencies.map(f => f.frequency),
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#ff4444' },
              { offset: 1, color: '#cc0000' }
            ])
          },
          label: {
            show: true,
            position: 'top',
            fontSize: 10,
            formatter: (params: any) => {
              const ball = redFrequencies[params.dataIndex];
              const weightPercent = totalWeight > 0 ? (ball.weight / totalWeight * 100).toFixed(1) : '0.0';
              return `${params.value}\n${weightPercent}%`;
            }
          }
        }]
      };
      chart.setOption(option, true); // 第二个参数设为 true，不合并配置，直接替换
      
      const handleResize = () => chart.resize();
      window.addEventListener('resize', handleResize);
      return () => {
        window.removeEventListener('resize', handleResize);
        chart.dispose();
      };
    }
  }, [redFrequencies]);

  // 渲染蓝球柱形图
  useEffect(() => {
    if (blueFrequencies.length > 0 && blueBarChartRef.current) {
      // 获取或创建图表实例
      let chart = echarts.getInstanceByDom(blueBarChartRef.current);
      if (!chart) {
        chart = echarts.init(blueBarChartRef.current, null, {
          renderer: 'canvas',
          useDirtyRect: true
        });
      }
      
      // 计算总权重用于百分比显示
      const totalWeight = blueFrequencies.reduce((sum, f) => sum + f.weight, 0);
      
      const option = {
        animation: false,
        title: {
          text: '蓝球频率柱形图',
          left: 'center',
          textStyle: { fontSize: 16, fontWeight: 'bold' }
        },
        tooltip: {
          trigger: 'axis',
          axisPointer: { type: 'shadow' },
          formatter: (params: any) => {
            const dataIndex = params[0].dataIndex;
            const ball = blueFrequencies[dataIndex];
            const weightPercent = totalWeight > 0 ? (ball.weight / totalWeight * 100).toFixed(2) : '0.00';
            return `球号: ${ball.number.toString().padStart(2, '0')}<br/>` +
                   `出现次数: ${ball.frequency}<br/>` +
                   `权重: ${ball.weight.toFixed(2)}<br/>` +
                   `权重占比: ${weightPercent}%`;
          }
        },
        xAxis: {
          type: 'category',
          data: blueFrequencies.map(f => f.number.toString().padStart(2, '0')),
          axisLabel: { interval: 0, rotate: 0, fontSize: 12 }
        },
        yAxis: {
          type: 'value',
          name: '频次'
        },
        series: [{
          name: '出现次数',
          type: 'bar',
          data: blueFrequencies.map(f => f.frequency),
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#4285f4' },
              { offset: 1, color: '#1a73e8' }
            ])
          },
          label: {
            show: true,
            position: 'top',
            fontSize: 10,
            formatter: (params: any) => {
              const ball = blueFrequencies[params.dataIndex];
              const weightPercent = totalWeight > 0 ? (ball.weight / totalWeight * 100).toFixed(1) : '0.0';
              return `${params.value}\n${weightPercent}%`;
            }
          }
        }]
      };
      chart.setOption(option, true); // 第二个参数设为 true，不合并配置，直接替换
      
      const handleResize = () => chart.resize();
      window.addEventListener('resize', handleResize);
      return () => {
        window.removeEventListener('resize', handleResize);
        chart.dispose();
      };
    }
  }, [blueFrequencies]);

  const getRecentRecords = () => {
    return records.slice(-10).reverse();
  };

  return (
    <main className="min-h-screen p-5 max-w-[1400px] mx-auto">
      <h1 className="text-center text-white text-4xl mb-8 drop-shadow-[2px_2px_4px_rgba(0,0,0,0.3)]">🎱 双色球预测工具</h1>
      {error && <div className="bg-red-500 text-white p-4 rounded-lg mb-5 text-center">{error}</div>}
      <div className="flex gap-5 items-center justify-center mb-8 flex-wrap">
        <button 
          onClick={loadData} 
          disabled={loading}
          className="px-8 py-3 text-base font-semibold border-2 border-white rounded-lg bg-white text-[#667eea] cursor-pointer transition-all duration-300 shadow-[0_4px_6px_rgba(0,0,0,0.1)] hover:transform hover:-translate-y-0.5 hover:shadow-[0_6px_12px_rgba(0,0,0,0.2)] disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
        >
          {loading ? "加载中..." : "刷新数据"}
        </button>
        <div className="flex gap-2.5 items-center">
          <label htmlFor="algorithm-select" className="text-white font-semibold">算法选择：</label>
          <select 
            id="algorithm-select"
            value={algorithm} 
            onChange={handleAlgorithmChange}
            disabled={loading}
            className="px-5 py-2.5 text-base font-semibold border-2 border-white rounded-lg bg-white text-[#667eea] cursor-pointer transition-all duration-300 shadow-[0_4px_6px_rgba(0,0,0,0.1)] outline-none hover:transform hover:-translate-y-0.5 hover:shadow-[0_6px_12px_rgba(0,0,0,0.2)] hover:border-[#764ba2] focus:border-[#764ba2] disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <option value="hot">热号恒热</option>
            <option value="cold">冷号反弹</option>
          </select>
        </div>
        <button
          onClick={generatePredictions}
          disabled={loading || records.length === 0}
          className="px-8 py-3 text-base font-semibold border-2 border-white rounded-lg bg-gradient-to-r from-[#667eea] to-[#764ba2] text-white cursor-pointer transition-all duration-300 shadow-[0_4px_6px_rgba(0,0,0,0.1)] hover:transform hover:-translate-y-0.5 hover:shadow-[0_6px_12px_rgba(0,0,0,0.2)] hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
        >
          {loading ? "生成中..." : "生成预测"}
        </button>
      </div>
      {records.length > 0 && (
        <div className="text-center text-white/90 bg-black/20 p-4 rounded-lg mt-8 text-sm backdrop-blur-[10px]">
          历史数据：{records.length} 期 | 最新期号：{records[records.length - 1]?.issue} | 
          日期：{records[records.length - 1]?.date}
        </div>
      )}
      {records.length > 0 && (
        <div className="bg-white/95 p-8 rounded-2xl mb-8 shadow-[0_8px_32px_rgba(0,0,0,0.1)] transform-gpu">
          <h2 className="text-[#333] text-2xl mb-5">📊 最近10期历史记录</h2>
          <div className="overflow-x-auto will-change-scroll">
            <table className="w-full border-collapse bg-white rounded-lg overflow-hidden min-w-[900px]">
              <thead className="bg-gradient-to-r from-[#667eea] to-[#764ba2] text-white sticky top-0 z-10">
                <tr>
                  <th className="p-4 text-left font-semibold text-sm">期号</th>
                  <th className="p-4 text-left font-semibold text-sm">开奖日期</th>
                  <th className="p-4 text-center font-semibold text-sm">开奖号码</th>
                </tr>
              </thead>
              <tbody>
                {getRecentRecords().map((record) => (
                  <tr key={record.issue} className="border-b border-[#f0f0f0] hover:bg-[#f8f9fa] transition-colors last:border-b-0">
                    <td className="p-4 text-sm font-semibold text-[#667eea]">{record.issue}</td>
                    <td className="p-4 text-sm text-[#666]">{record.date}</td>
                    <td className="p-4 text-sm">
                      <div className="flex gap-2 flex-nowrap whitespace-nowrap justify-center">
                        {record.red_balls.map((ball, idx) => (
                          <span 
                            key={idx} 
                            className="w-9 h-9 rounded-full bg-gradient-to-br from-red-500 to-red-600 text-white flex items-center justify-center text-sm font-bold shadow-md flex-shrink-0"
                          >
                            {ball.toString().padStart(2, '0')}
                          </span>
                        ))}
                        <span className="text-2xl font-bold text-[#999] mx-1">+</span>
                        <span className="w-9 h-9 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 text-white flex items-center justify-center text-sm font-bold shadow-md flex-shrink-0">
                          {record.blue_ball.toString().padStart(2, '0')}
                        </span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
      {redFrequencies.length > 0 && (
        <div className="bg-white/95 p-8 rounded-2xl mb-8 shadow-[0_8px_32px_rgba(0,0,0,0.1)] transform-gpu">
          <h2 className="text-[#333] text-2xl mb-5">🔴 红球频率分析</h2>
          
          {/* 柱形图 */}
          <div className="my-5 p-5 bg-white rounded-xl shadow-[0_2px_8px_rgba(0,0,0,0.05)] transform-gpu">
            <div ref={redBarChartRef} style={{ width: '100%', height: '400px' }} className="will-change-transform"></div>
          </div>
        </div>
      )}
      {blueFrequencies.length > 0 && (
        <div className="bg-white/95 p-8 rounded-2xl mb-8 shadow-[0_8px_32px_rgba(0,0,0,0.1)] transform-gpu">
          <h2 className="text-[#333] text-2xl mb-5">🔵 蓝球频率分析</h2>
          
          {/* 柱形图 */}
          <div className="my-5 p-5 bg-white rounded-xl shadow-[0_2px_8px_rgba(0,0,0,0.05)] transform-gpu">
            <div ref={blueBarChartRef} style={{ width: '100%', height: '400px' }} className="will-change-transform"></div>
          </div>
        </div>
      )}
      {predictions.length > 0 && (
        <div className="bg-white/95 p-8 rounded-2xl mb-8 shadow-[0_8px_32px_rgba(0,0,0,0.1)] transform-gpu">
          <h2 className="text-[#333] text-2xl mb-5">🎯 推荐号码（前10组）</h2>
          <div className="grid gap-4 md:grid-cols-2">
            {predictions.map((pred, index) => (
              <div key={index} className="flex items-center gap-4 p-5 bg-white rounded-xl shadow-[0_2px_8px_rgba(0,0,0,0.05)] transition-all duration-300 hover:shadow-[0_4px_16px_rgba(0,0,0,0.1)] hover:transform hover:-translate-y-1 will-change-transform">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white flex items-center justify-center text-lg font-bold shadow-lg flex-shrink-0">
                  #{index + 1}
                </div>
                <div className="flex items-center gap-2.5 flex-1 flex-wrap">
                  <div className="flex gap-1.5">
                    {pred.red_balls.map((ball, i) => (
                      <span 
                        key={i} 
                        className="w-10 h-10 rounded-full bg-gradient-to-br from-red-500 to-red-600 text-white flex items-center justify-center text-sm font-bold shadow-md transition-all duration-300 hover:transform hover:scale-110 will-change-transform"
                      >
                        {ball.toString().padStart(2, '0')}
                      </span>
                    ))}
                  </div>
                  <span className="text-2xl font-bold text-[#999] mx-1">+</span>
                  <div className="flex gap-1.5">
                    <span className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 text-white flex items-center justify-center text-sm font-bold shadow-md transition-all duration-300 hover:transform hover:scale-110 will-change-transform">
                      {pred.blue_ball.toString().padStart(2, '0')}
                    </span>
                  </div>
                </div>
                <div className="text-sm text-[#666] whitespace-nowrap ml-auto">
                  得分: <span className="font-bold text-[#667eea]">{pred.score.toFixed(2)}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
      <footer className="text-center text-white/90 bg-black/20 p-4 rounded-lg mt-8 text-sm backdrop-blur-[10px]">
        ⚠️ 本工具仅供娱乐参考，不构成任何投资建议
      </footer>
    </main>
  );
}

export default App;
