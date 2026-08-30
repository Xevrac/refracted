using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GameplayEffectStatusUpdate
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GameplayEffectStatusUpdate); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GameplayEffectStatusUpdate)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize EffectType
            s.Write(value.EffectType);
            //  Serialize Enabled
            s.Write(value.Enabled);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GameplayEffectStatusUpdate)) as Rts.CnC.Messages.Client.GameplayEffectStatusUpdate;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize EffectType
            s.Read(out value.EffectType);
            //  Deserialize Enabled
            s.Read(out value.Enabled);

            return value;
        }
        
    }
}
