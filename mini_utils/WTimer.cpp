
#include "Log.h"
#include "Macro.h"

#include "WTimer.h"

using namespace MSA;

//第1个轮子槽0~255
#define TVR_MSAK (TVR_SIZE - 1)

//第2~5轮子槽0~63
#define TVN_MSAK (TVN_SIZE - 1)


WTimer::WTimer(uint16_t tickSize)
{
	m_Wheel.curTick = 0;
	for (auto i = 0; i < TVR_SIZE; i++)
	{
		auto entry = m_Wheel.tv1 + i;
		entry->head = entry->tail = nullptr;
	}

	for (auto i = 0; i < TVN_SIZE; i++)
	{
		auto entry = m_Wheel.tv2 + i;
		entry->head = entry->tail = nullptr;
		entry = m_Wheel.tv3 + i;
		entry->head = entry->tail = nullptr;
		entry = m_Wheel.tv4 + i;
		entry->head = entry->tail = nullptr;
		entry = m_Wheel.tv5 + i;
		entry->head = entry->tail = nullptr;

	}
	m_TickSize = tickSize < 1 ? 1 : tickSize;
}

WTimer::~WTimer()
{
	DelTvTimer(m_Wheel.tv1, TVR_SIZE);
	DelTvTimer(m_Wheel.tv2, TVN_SIZE);
	DelTvTimer(m_Wheel.tv3, TVN_SIZE);
	DelTvTimer(m_Wheel.tv4, TVN_SIZE);
	DelTvTimer(m_Wheel.tv5, TVN_SIZE);
	if (nullptr != m_CacheTimerArray)
	{
		SAFE_DELETE_ARRAY(m_CacheTimerArray);
	}
}

void WTimer::DelTvTimer(List* tv, uint16_t size)
{
	for (auto i = 0; i < size; i++)
	{
		List* entry = (tv + i);
		auto curr = entry->head;
		while (nullptr != curr)
		{
			Timer* next = curr->next;
			if (curr->id > m_CacheTimerNum)
			{
				SAFE_DELETE(curr);
			}
			curr = next;
		}
		entry->head = entry->tail = nullptr;
	}
}


bool WTimer::Init(uint32_t cacheTimerNum)
{
	//已初始化
	if (m_Wheel.curTick > 0) return false;
	auto nowTime = SysTime::GetSysMilSecond();
	m_Wheel.curTick = nowTime / m_TickSize;

	if (0 == cacheTimerNum)  return false;
	if (cacheTimerNum < 1)
	{
		m_CacheTimerNum = 1024;
	}
	else
	{
		m_CacheTimerNum = cacheTimerNum;
	}

	return InitCacheTimer(m_CacheTimerNum);
}

uint32_t WTimer::AddTimer(const std::function<void(uint64_t)> fun, uint64_t data, uint32_t delay, uint32_t interval, int16_t runNum)
{
	auto timer = (Timer*)m_CustomStack.Pop();
	if (nullptr == timer)
	{
		timer = new(nothrow) Timer;
		if (nullptr == timer)
		{
			LogError("not enough memory");
			return 0;
		}
		timer->id = GetNewTimerId();
	}
	timer->fun = fun;
	timer->data = data;
	timer->runNum = runNum == 0 ? 1 : runNum;
	timer->interval = interval < 0 ? 1 : interval;

	if (delay < 0) delay = 0;
	timer->expire = SysTime::GetSysMilSecond() + delay;
	m_TimerMap.insert(pair<uint32_t, Timer*>(timer->id, timer));

	AddTimer(timer); return timer->id;

}

void WTimer::AddTimer(Timer* timer)
{
	//过期时刻度
	auto ticks = timer->expire / m_TickSize;
	uint64_t idx = ticks - m_Wheel.curTick;

	List* entry;
	auto i = 0;
	if (idx < TVR_SIZE) {
		i = ticks & TVR_MSAK;
		entry = m_Wheel.tv1 + i;
	}
	else if (idx < 1 << (TVR_BITS + TVN_BITS)) {
		i = (ticks >> TVR_BITS) & TVN_MSAK;
		entry = m_Wheel.tv2 + i;
	}
	else if (idx < 1 << (TVR_BITS + 2 * TVN_BITS)) {
		i = (ticks >> (TVR_BITS + TVN_BITS)) & TVN_MSAK;
		entry = m_Wheel.tv3 + i;
	}
	else if (idx < 1 << (TVR_BITS + 3 * TVN_BITS)) {
		i = (ticks >> (TVR_BITS + 2 * TVN_BITS)) & TVN_MSAK;
		entry = m_Wheel.tv4 + i;
	}
	else if ((signed long)idx < 0) {
		entry = m_Wheel.tv1 + (m_Wheel.curTick & TVR_MSAK);
	}
	else {
		if (idx > 0xffffffffUL) {
			idx = 0xffffffffUL;
			ticks = idx + m_Wheel.curTick;
		}
		i = (ticks >> (TVR_BITS + 3 * TVN_BITS)) & TVN_MSAK;
		entry = m_Wheel.tv5 + i;
	}
	if (nullptr != entry->tail)
	{
		timer->prev = entry->tail;
		entry->tail->next = timer;
		entry->tail = timer;
		timer->entry = entry;
		timer->next = nullptr;
	}
	else
	{
		timer->entry = entry;
		entry->head = entry->tail = timer;
		timer->prev = timer->next = nullptr;
	}

}


void WTimer::Scheduled()
{
	Scheduled(SysTime::GetSysMilSecond());
}

uint8_t WTimer::Cascade(List* tv, uint8_t idx)
{
	auto entry = tv + idx;
	auto curr = entry->head;
	entry->head = nullptr;
	entry->tail = nullptr;
	while (nullptr != curr)
	{
		auto next = curr->next;
		AddTimer(curr); curr = next;
	}
	return idx;
}

#define INDEX(N) ((m_Wheel.curTick >> (TVR_BITS + (N) * TVN_BITS)) & TVN_MSAK)

//currTime:调度时间要大于StartTime单位(毫秒)
void WTimer::Scheduled(uint64_t currTime)
{
	auto currTick = currTime / m_TickSize;
	while (m_Wheel.curTick < currTick)
	{
		auto idx = m_Wheel.curTick & TVR_MSAK;

		if (!idx &&
			(!Cascade(m_Wheel.tv2, INDEX(0))) &&
			(!Cascade(m_Wheel.tv3, INDEX(1))) &&
			!Cascade(m_Wheel.tv4, INDEX(2)))
		{
			Cascade(m_Wheel.tv5, INDEX(3));
		}

		m_Wheel.curTick++;

		auto entry = m_Wheel.tv1 + idx;
		auto currTimer = entry->head;
		if (nullptr != currTimer)
		{
			m_Wheel.runningTimer = currTimer;
			//清空当前槽
			entry->head = entry->tail = nullptr;
			do
			{
				auto next = currTimer->next;
				currTimer->fun(currTimer->data);
				if (currTimer->runNum > 0)
				{
					currTimer->runNum--;
				}
				if (0 == currTimer->runNum)
				{
					DelTimer(currTimer);//调用完就删除
				}
				else
				{
					//会延时一毫秒运行
					//秒为tick会晚一秒 减一毫秒就不会晚一秒
					auto nowTime = SysTime::GetSysMilSecond() - 1;
					currTimer->expire = nowTime + currTimer->interval;
					AddTimer(currTimer);
				}

				m_Wheel.runningTimer = currTimer = next;
			} while (nullptr != currTimer);
		}
	}
}

uint32_t WTimer::GetNewTimerId()
{
	m_LastTimerId++;
	if (m_LastTimerId <= m_CacheTimerNum)
	{
		m_LastTimerId = m_CacheTimerNum + 1;
	}
	while (m_TimerMap.find(m_LastTimerId) != m_TimerMap.end())
	{
		m_LastTimerId++;
		if (m_LastTimerId <= m_CacheTimerNum)
		{
			m_LastTimerId = m_CacheTimerNum + 1;
		}
	}
	return m_LastTimerId;
}

//真正删除Timer
void WTimer::DelTimer(Timer* timer)
{
	m_TimerMap.erase(timer->id);

	if (timer->id > m_CacheTimerNum)
	{
		SAFE_DELETE(timer);
	}
	else
	{
		m_CustomStack.Push(timer);
	}
}


void WTimer::DelTimer(uint32_t id)
{
	if (nullptr != m_Wheel.runningTimer)
	{
		if (m_Wheel.runningTimer->id == id)
		{
			//调用完后 会减1 为0删除
			m_Wheel.runningTimer->runNum = 1;
			return;
		}
	}

	auto iter = m_TimerMap.find(id);
	if (iter != m_TimerMap.end())
	{
		//DelTimer(iter->second);
		auto timer = iter->second;
		//非链表头部
		if (nullptr != timer->prev)
		{
			timer->prev->next = timer->next;
		}
		else
		{
			auto entry = timer->entry;
			entry->head = timer->next;
			if (timer->next == nullptr)
			{
				entry->tail = nullptr;
			}
		}
		DelTimer(timer);
	}
}

bool WTimer::InitCacheTimer(uint16_t cacheNum)
{
	m_CacheTimerArray = new(nothrow) Timer[cacheNum];
	if (nullptr == m_CacheTimerArray)
	{
		LogError("not enough memory");
		return  false;
	}

	if (!m_CustomStack.Init(cacheNum))
	{
		LogError("not enough memory");
		return  false;
	}

	for (auto i = 0; i < cacheNum; i++)
	{
		m_CacheTimerArray[i].id = i + 1;
		m_CustomStack.Push(&m_CacheTimerArray[i]);
	}
	return true;
}
