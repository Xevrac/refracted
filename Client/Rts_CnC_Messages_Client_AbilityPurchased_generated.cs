using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityPurchased
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityPurchased); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityPurchased)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);
            //  Serialize ConsumablePower
            s.Write(value.ConsumablePower);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityPurchased)) as Rts.CnC.Messages.Client.AbilityPurchased;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);
            //  Deserialize ConsumablePower
            s.Read(out value.ConsumablePower);

            return value;
        }
        
    }
}
