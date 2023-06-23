import { createApp, ref, computed, watch } from 'vue'
import dayjs from 'dayjs'

createApp({
  template: /* html */ `
    <div class="flex flex-col w-full">
      <!-- 顶部 -->
      <div class="flex flex-col bg-gray-100">
        <div class="flex flex-col py-12 gap-8 w-4/5 mx-auto max-w-screen-lg">
          <!-- logo -->
          <div class="flex flex-col w-full">
            <img src="/images/szu.svg" alt="logo" class="w-128 h-36" />
          </div>

          <!-- 搜索框 -->
          <div class="relative flex w-full h-12 rounded-full px-6 shadow-lg bg-white">
            <input
              class="peer h-full w-full outline-none text-md text-gray-700"
              type="text"
              v-model="keyword"
              placeholder="输入你想搜索的内容"
              @keyup.enter="onClickSearch"
            />
            <div @click="onClickSearch" class="cursor-pointer grid place-items-center w-12 h-12 text-gray-400">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
          </div>

          <!-- 筛选 -->
          <div class="flex flex-col gap-4 w-full">
            <div class="flex flex-row gap-3 items-center text-sm">
              <span class="font-bold">类别</span>
                <span
                  v-for="v,i in allInfotypes" :key="i" @click="selectedInfotype = i"
                  :class="['px-3 leading-6 rounded-full cursor-pointer', selectedInfotype === i ? 'text-white bg-blue-600' : 'text-gray-600 bg-gray-200']"
                >
                  {{ v }}
                </span>
            </div>
            <div class="flex flex-row flex-wrap gap-3 items-center text-sm">
              <span class="font-bold">单位</span>
              <span
                v-for="v,i in allUsers" :key="i" @click="selectedUser = i"
                :class="['px-3 leading-6 rounded-full cursor-pointer', selectedUser === i ? 'text-white bg-blue-600' : 'text-gray-600 bg-gray-200']"
              >
                {{ v }}
              </span>
            </div>
          </div>
              
        </div>
      </div>

      <!-- 搜索结果 -->
      <div class="flex flex-col items-center">
        <div v-if="result" class="flex flex-col py-8 gap-8 w-4/5 mx-auto max-w-screen-lg">
          <p class="text-sm text-gray-500">找到 <b>{{ result.total_hits }}</b> 条结果 （用时 <b>{{ result.time }}</b> 毫秒）</p>

          <p v-if="result.total_hits === 0" class="text-md text-gray-600">找不到和您查询的“{{ result.keyword }}”相符的内容或信息。</p>

          <div v-for="hit in result.hits" :key="hit.id" class="flex flex-col gap-2">
            <!-- 信息 -->
            <div class="flex flex-row items-center gap-4">
              <!-- 标题 -->
              <a :href="hit.doc.url" target="_blank">
                <span class="text-lg text-blue-700">{{ formatTitle(hit.doc.title) }}</span>
              </a>
              <!-- 得分 -->
              <span class="text-xl text-red-700">{{ hit.score.toFixed(2) }}</span>
              <!-- 图文按钮 -->
              <button @click="hit.raw = hit.raw ? false : true" :class="['px-3 py-1 rounded-full text-sm cursor-pointer', hit.raw ? 'text-white bg-blue-600' : 'text-gray-600 bg-gray-200']">图文</button>
              <!-- 时间 -->
              <span class="ml-auto text-sm text-gray-400">{{ formatTime(hit.doc.time) }}</span>
            </div>

            <!-- 内容 -->
            <div class="text-sm text-gray-500">
              <div v-if="hit.raw" class="border h-96 rounded-md overflow-scroll">
                <article class="szu m-2" v-html="hit.doc.html" />
              </div>
              <p v-else class="line-clamp-3">{{ hit.doc.text }}</p>
            </div>

            <!-- 附件 -->
            <div v-if="hit.doc.attachments.length > 0" class="flex flex-col text-sm text-red-500 leading-6">
              <a v-for="attachment in hit.doc.attachments" :href="attachment.url" target="_blank">
                <span class="flex flex-row items-center gap-4">{{ attachment.name }}</span>
              </a>
            </div>
          </div>

          <!-- 翻页按钮 -->
          <div v-if="result.hits.length > 0" class="flex flex-row h-12 justify-center items-center gap-4">
            <div @click="onClickPrev" :class="['px-3 py-1 rounded-full text-sm', hasPrevPage ? 'text-white bg-blue-600 cursor-pointer' : 'text-gray-400 bg-gray-100']">上一页</div>
            <div @click="onClickNext" :class="['px-3 py-1 rounded-full text-sm', hasNextPage ? 'text-white bg-blue-600 cursor-pointer' : 'text-gray-400 bg-gray-100']">下一页</div>
          </div>
        </div>
      </div>
    </div>
  `,
  setup() {
    const allInfotypes = [
      '全部',
      '教务',
      '科研',
      '行政',
      '学工',
      '会议',
      '讲座',
      '生活',
    ]
    const allUsers = [
      '全部',
      '党政办公室',
      '组织部',
      '统战部',
      '宣传部',
      '纪检（监察）室',
      '校工会',
      '妇女委员会',
      '校团委',
      '教务部',
      '招生办公室',
      '创新创业教育中心',
      '继续教育管理办公室',
      '研究生院',
      '党委研工部',
      '发展规划部',
      '社会科学部',
      '学报社科版',
      '科学技术部',
      '学报理工版',
      '学生部',
      '党委学工部',
      '国际交流与合作部',
      '人力资源部',
      '党委教师工作部',
      '计划财务部',
      '招投标管理中心',
      '实验室与国有资产管理部',
      '审计室',
      '后勤保障部',
      '后勤保障部党委',
      '安全保卫部',
      '离退休办公室',
      '校友联络部',
      '教育发展基金会',
      '机关党委',
      '丽湖校区管理办公室',
      '教育学部',
      '艺术学部',
      '医学部',
      '马克思主义学院',
      '经济学院',
      '法学院',
      '心理学院',
      '体育学院',
      '人文学院',
      '外国语学院',
      '传播学院',
      '数学与统计学院',
      '物理与光电工程学院',
      '化学与环境工程学院',
      '生命与海洋科学学院',
      '机电与控制工程学院',
      '材料学院',
      '电子与信息工程学院',
      '计算机与软件学院',
      '建筑与城市规划学院',
      '土木与交通工程学院',
      '管理学院',
      '政府管理学院',
      '高等研究院',
      '金融科技学院',
      '国际交流学院',
      '继续教育学院',
      '图书馆',
      '图书馆党总支',
      '档案馆',
      '信息中心',
      '信息中心党总支',
      '资产经营公司',
      '技术转化中心',
      '深大总医院',
      '深大附属华南医院',
      '校医院',
      '附属教育集团',
      '深大附属中学',
      '深大附属实验中学',
      '中国经济特区研究中心',
      '港澳基本法研究中心',
      '文化产业研究院',
      '美学与文艺批评研究院',
      '饶宗颐文化研究院',
      '中国海外利益研究院',
      '微纳光电子学研究院',
      '创新技术研究院',
      '大数据系统计算技术国家工程实验室',
      '心理健康教育与咨询中心',
      '人工智能与数字经济广东省实验室（深圳）',
      '深圳香蜜湖国际金融科技研究院',
    ]

    const keyword = ref('')
    const selectedInfotype = ref(0)
    const selectedUser = ref(0)
    const offset = ref(0)
    const limit = ref(10)
    const result = ref()
    const searching = ref(false)

    const hasNextPage = computed(
      () => result.value && result.value.total_hits > offset.value + limit.value
    )
    const hasPrevPage = computed(() => offset.value > 0)

    const doSearch = async (first) => {
      if (first) {
        offset.value = 0
      }

      if (keyword.value) {
        searching.value = true
        const res = await fetch('/search', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            keyword: keyword.value,
            offset: offset.value,
            limit: limit.value,
            filter: {
              infotype:
                selectedInfotype.value === 0
                  ? undefined
                  : allInfotypes[selectedInfotype.value],
              user:
                selectedUser.value === 0
                  ? undefined
                  : allUsers[selectedUser.value],
            },
          }),
        })
        result.value = await res.json()
        searching.value = false
        window.scrollTo(0, 0)
      }
    }

    watch([selectedInfotype, selectedUser], () => {
      doSearch(true)
    })

    const onClickSearch = () => doSearch(true)

    const onClickNext = () => {
      if (!hasNextPage.value) return
      offset.value += limit.value
      doSearch(false)
    }

    const onClickPrev = () => {
      if (!hasPrevPage.value) return
      offset.value = Math.max(offset.value - limit.value, 0)
      doSearch(false)
    }

    const formatTitle = (title) => {
      if (title.length > 30) {
        return title.slice(0, 30) + '...'
      } else {
        return title
      }
    }

    const formatTime = (time) => {
      return dayjs(time).format('YYYY年MM月DD日 HH:mm:ss')
    }

    return {
      allInfotypes,
      allUsers,
      keyword,
      selectedInfotype,
      selectedUser,
      offset,
      limit,
      result,
      searching,
      hasNextPage,
      hasPrevPage,
      onClickSearch,
      onClickNext,
      onClickPrev,
      formatTitle,
      formatTime,
    }
  },
}).mount('#app')
