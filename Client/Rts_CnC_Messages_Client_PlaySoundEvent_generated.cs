using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlaySoundEvent
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlaySoundEvent); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlaySoundEvent)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize EventId
            s.Write(value.EventId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlaySoundEvent)) as Rts.CnC.Messages.Client.PlaySoundEvent;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize EventId
            s.Read(out value.EventId);

            return value;
        }
        
    }
}
