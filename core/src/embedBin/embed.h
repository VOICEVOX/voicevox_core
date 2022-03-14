#ifndef __EMBED_H
#define __EMBED_H
#ifndef EMBED_DATA_NAME
#define EMBED_DATA_NAME data
#endif
#ifndef EMBED_SIZE_NAME
#define EMBED_SIZE_NAME size
#endif
#ifndef EMBED_NS
#define EMBED_NS embed
#endif
#ifndef EMBED_STRUCT
#ifdef __cplusplus
#define EMBED_RES Resource
#else
#define EMBED_RES embed_resource
#endif
#endif
#ifndef EMBED_RES_TYPE
#define EMBED_RES_TYPE embed_resource_t
#endif
#ifdef __cplusplus
#include <cstddef>
#define EMBED_DECL(NAME) extern "C" EMBED_NS::EMBED_RES NAME(void)
namespace EMBED_NS {
struct EMBED_RES {
	const char *EMBED_DATA_NAME;
	std::size_t EMBED_SIZE_NAME;
};
}
#else
#include <stddef.h>
#define EMBED_DECL(NAME) extern struct EMBED_RES NAME(void)
typedef struct EMBED_RES {
	const char *EMBED_DATA_NAME;
	size_t EMBED_SIZE_NAME;
} EMBED_RES_TYPE;
#endif
#endif