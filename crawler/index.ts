import axios from 'axios'
import * as cheerio from 'cheerio'
import iconv from 'iconv-lite'
import fs from 'fs'

const COOKIE = '<COOKIE>'
const output = '../dataset'

function encodeURIComponent_GBK(str: string) {
  const splitted = str.toString().split('')
  for (let i = 0; i < splitted.length; i++) {
    const c = splitted[i]
    // https://stackoverflow.com/questions/695438/what-are-the-safe-characters-for-making-urls
    // ALPHA / DIGIT / "-" / "." / "_" / "~" / "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "=" / ":" / "@"
    if (/[a-zA-Z0-9\-._~!$&'()*+,;=:@]/.test(c)) {
      continue // 不编码
    }

    const buffer = iconv.encode(c, 'gbk')
    const encoded = [''] // 注意先放个空字符串，保证 join 后最前面有一个%
    for (let j = 0; j < buffer.length; j++) {
      encoded.push(buffer.toString('hex', j, j + 1).toUpperCase())
    }
    splitted[i] = encoded.join('%')
  }
  return splitted.join('')
}

interface Info {
  infotype: string
  user: string
  id: number
}

async function infoList(year: number, user: string): Promise<Info[]> {
  const res = await axios({
    method: 'POST',
    url: 'https://www1.szu.edu.cn/board/infolist.asp',
    data: [
      `dayy=${year}`,
      `from_username=${encodeURIComponent_GBK(user)}`,
      `keyword=`,
      `searchb1=${encodeURIComponent_GBK('搜索')}`,
    ].join('&'),
    headers: {
      'content-type': 'application/x-www-form-urlencoded',
      cookie: COOKIE,
    },
    responseType: 'arraybuffer',
  })

  const html = iconv.decode(res.data, 'gbk')
  const $ = cheerio.load(html)
  const table = $(
    'body > table > tbody > tr:nth-child(2) > td > table > tbody > tr:nth-child(3) > td > table > tbody > tr:nth-child(3) > td > table'
  )
  const rows = table.find('tr')

  const infos: Info[] = []
  rows.slice(2, 42).each((_, tr) => {
    const infotype = $(tr).find('td:nth-child(2) > a').text()
    const user = $(tr).find('td:nth-child(3) > a').text()
    const id = parseInt(
      $(tr).find('td:nth-child(4) > a').attr('href')!.split('=')[1]
    )
    infos.push({ infotype, user, id })
  })
  return infos
}

interface Doc {
  url: string
  infotype: string
  user: string
  title: string
  text: string
  html: string
  time: Date
  // 注意，附件的下载需要 `Referer: https://www1.szu.edu.cn` 和 `Cookie: <COOKIE>`
  attachments: Attachment[]
}

interface Attachment {
  name: string
  url: string
}

async function infoDetail(info: Info): Promise<Doc> {
  const url = `https://www1.szu.edu.cn/board/view.asp?id=${info.id}`
  const res = await axios({
    method: 'GET',
    url,
    headers: {
      cookie: COOKIE,
    },
    responseType: 'arraybuffer',
  })

  const html = iconv.decode(res.data, 'gbk')
  const $ = cheerio.load(html)

  const tableSelector =
    '#bodyBoard > table > tbody > tr:nth-child(2) > td > table > tbody > tr:nth-child(3) > td > table > tbody > tr:nth-child(2) > td > table'

  const doc = {
    url,
    infotype: info.infotype,
    user: info.user,
    title: $(`${tableSelector} > tbody > tr:nth-child(1) > td`).text().trim(),
    text: $(`${tableSelector} > tbody > tr:nth-child(3) > td`).text().trim(),
    html: (() => {
      let content = $(`${tableSelector} > tbody > tr:nth-child(3) > td`)
      content.find('img').each((_, img) => {
        if ($(img).attr('src')?.startsWith('/')) {
          $(img).attr('src', `https://www1.szu.edu.cn${$(img).attr('src')}`)
        }
      })
      return content.html()!
    })(),
    time: new Date(
      $(`${tableSelector} > tbody > tr:nth-child(2) > td > font:nth-child(1)`)
        .text()
        .split('　')[1]
    ),
    attachments: $(`${tableSelector} > tbody > tr:nth-child(4) > td`)
      .children('a')
      .map((_, a) => {
        const fn = encodeURIComponent_GBK($(a).attr('href')!.split('=')[1])
        const name = $(a).text().trim()
        const url = `https://www1.szu.edu.cn/board/down1oad.asp?fn=${fn}`
        return {
          name,
          url,
        }
      })
      .get(),
  }

  if (doc.time.toString() === 'Invalid Date') {
    throw new Error(
      `Invalid doc at ${url}, d: ${$(
        `${tableSelector} > tbody > tr:nth-child(2) > td > font`
      ).text()}}`
    )
  }

  return doc
}

async function main() {
  const years = [
    2023, 2022, 2021, 2020, 2019, 2018, 2017, 2016, 2015, 2014, 2013, 2012,
    2011, 2010, 2009, 2008, 2007, 2006, 2005, 2004, 2003, 2002,
  ]
  const users = [
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

  if (!fs.existsSync(output)) {
    fs.mkdirSync(output)
  }

  for (const year of years) {
    for (const user of users) {
      if (!fs.existsSync(`${output}/${year}/${user}`)) {
        fs.mkdirSync(`${output}/${year}/${user}`, { recursive: true })
      }

      const infos = await infoList(year, user)
      const data = await Promise.all(infos.map((info) => infoDetail(info)))
      console.log(`爬取 ${year}-${user} 完成`)

      await Promise.all(
        data.map((doc, i) =>
          fs.promises.writeFile(
            `${output}/${year}/${user}/${infos[i].id}.json`,
            JSON.stringify(doc, null, 2)
          )
        )
      )
    }
  }
}

main()
