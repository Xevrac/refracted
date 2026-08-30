using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EffectStarted
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EffectStarted); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EffectStarted)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize BuffStatus
            s.Write(value.BuffStatus);
            //  Serialize BuffCategory
            s.Write(value.BuffCategory);
            //  Serialize EffectId
            s.Write(value.EffectId);
            //  Serialize UniqueId
            s.Write(value.UniqueId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EffectStarted)) as Rts.CnC.Messages.Client.EffectStarted;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize BuffStatus
            s.Read(out value.BuffStatus);
            //  Deserialize BuffCategory
            s.Read(out value.BuffCategory);
            //  Deserialize EffectId
            s.Read(out value.EffectId);
            //  Deserialize UniqueId
            s.Read(out value.UniqueId);

            return value;
        }
        
    }
}
