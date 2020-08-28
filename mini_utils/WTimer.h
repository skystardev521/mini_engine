#ifndef WTIMER_H
#define WTIMER_H

#include <stdint.h>
#include <functional>
#include <unordered_map>

#include "SysTime.h"
#include "CustomStack.h"


//第1个轮子占用8位
#define TVR_BITS 8
//第2~5轮子各占用6位
#define TVN_BITS 6

//第1个轮子槽数量为256
#define TVR_SIZE (1 << TVR_BITS)
//第2~5轮子槽数量为64
#define TVN_SIZE (1 << TVN_BITS)

using namespace std;

#define BIND_CBFUN(callback) std::bind(callback, this, std::placeholders::_1)

namespace MSA
{

	class WTimer
	{
	private:

		struct List;

		struct Timer
		{
			//id定器Id
			uint32_t id;
			Timer* prev;
			Timer* next;
			List* entry;

			//fun的参数
			uint64_t data;
			//运行次数-1无限
			uint16_t runNum;
			//过期时间毫秒
			uint64_t expire;
			//每次运行后间隔时长
			uint32_t interval;
			//反回true 删除当前定时器
			function<void(uint64_t)> fun;
		};

		struct List
		{
			Timer* head;
			Timer* tail;
		};
		struct Wheel
		{
			//当前刻度
			uint64_t curTick;
			List tv1[TVR_SIZE];
			List tv2[TVN_SIZE];
			List tv3[TVN_SIZE];
			List tv4[TVN_SIZE];
			List tv5[TVN_SIZE];
			Timer* runningTimer;
		};

		Wheel m_Wheel;

		//每个Tick多少毫秒
		uint16_t m_TickSize;

		uint32_t m_CacheTimerNum;
		Timer* m_CacheTimerArray;
		CustomStack m_CustomStack;

		//每个Timer产生一个Id
		uint32_t m_LastTimerId = 0;

		//每AddTimer的都会存放到这里
		unordered_map<uint32_t, Timer*> m_TimerMap;

		uint32_t GetNewTimerId();

		void AddTimer(Timer* timer);
		void DelTimer(Timer* timer);

		bool InitCacheTimer(uint16_t cacheNum);

		uint8_t Cascade(List* tv, uint8_t idx);
		void DelTvTimer(List* tv, uint16_t size);

	public:

		~WTimer();
		//tickSize:最少间隔单位毫秒
		WTimer(uint16_t tickSize);
		//cacheTimerNum:默认创建Timer的数量
		bool Init(uint32_t cacheTimerNum);

		void Scheduled();
		//currTime当前时间单位毫秒
		void Scheduled(uint64_t currTime);

		//id Timer Id
		void DelTimer(uint32_t id);

		//增加一个定时器
		//delay:当前时间开始延时时长
		//interval:每次运行后间隔时长
		//num:运行次数-1:无限次，大于0
		//反回值:0没内存,大于0为Timer Id
		uint32_t AddTimer(const function<void(uint64_t)> fun, uint64_t data, uint32_t delay, uint32_t interval = 1, int16_t runNum = 1);

	};

}
#endif